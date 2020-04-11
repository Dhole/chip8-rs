#![no_std]

use core::ops::{Index, IndexMut};

use rand::rngs::SmallRng;
use rand::{RngCore, SeedableRng};

const SPRITE_CHARS: [[u8; 5]; 0x10] = [
    [0xF0, 0x90, 0x90, 0x90, 0xF0], // 0
    [0x20, 0x60, 0x20, 0x20, 0x70], // 1
    [0xF0, 0x10, 0xF0, 0x80, 0xF0], // 2
    [0xF0, 0x10, 0xF0, 0x10, 0xF0], // 3
    [0x90, 0x90, 0xF0, 0x10, 0x10], // 4
    [0xF0, 0x80, 0xF0, 0x10, 0xF0], // 5
    [0xF0, 0x80, 0xF0, 0x90, 0xF0], // 6
    [0xF0, 0x10, 0x20, 0x40, 0x40], // 7
    [0xF0, 0x90, 0xF0, 0x90, 0xF0], // 8
    [0xF0, 0x90, 0xF0, 0x10, 0xF0], // 9
    [0xF0, 0x90, 0xF0, 0x90, 0x90], // A
    [0xE0, 0x90, 0xE0, 0x90, 0xE0], // B
    [0xF0, 0x80, 0x80, 0x80, 0xF0], // C
    [0xE0, 0x90, 0x90, 0x90, 0xE0], // D
    [0xF0, 0x80, 0xF0, 0x80, 0xF0], // E
    [0xF0, 0x80, 0xF0, 0x80, 0x80], // F
];
const SPRITE_CHARS_ADDR: u16 = 0x0000;
pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGTH: usize = 32;
const MEM_SIZE: usize = 0x1000;
const ROM_ADDR: usize = 0x200;

#[derive(Debug)]
pub enum Error {
    InvalidOp(u8, u8),
    RomTooBig(usize),
    PcOutOfBounds(u16),
    Debug,
}

struct Reg(u8);

struct Regs([u8; 0x10]);

impl Index<u8> for Regs {
    type Output = u8;

    fn index(&self, reg: u8) -> &Self::Output {
        &self.0[reg as usize]
    }
}

impl IndexMut<u8> for Regs {
    fn index_mut(&mut self, reg: u8) -> &mut Self::Output {
        &mut self.0[reg as usize]
    }
}

impl Regs {
    fn new() -> Self {
        Self([0; 0x10])
    }
}

pub struct Chip8<R: RngCore> {
    mem: [u8; MEM_SIZE],
    v: Regs, // Register Set
    i: u16,
    pc: u16, // Program Counter
    stack: [u16; 0x10],
    sp: u8,                                         // Stack Pointer
    dt: u8,                                         // Delay Timer
    st: u8,                                         // Sound TImer
    pub k: u16,                                     // Keypad
    pub fb: [u8; SCREEN_WIDTH * SCREEN_HEIGTH / 8], // Framebuffer
    rng: R,
}

macro_rules! nnn {
    ($w0:expr, $w1:expr) => {
        (($w0 & 0x0f) as u16) << 8 | $w1 as u16
    };
}

pub struct Output {
    pub tone: bool,
    pub overtime: usize,
}

impl Chip8<SmallRng> {
    pub fn new(seed: u64) -> Self {
        let mut mem = [0; MEM_SIZE];
        for (i, sprite) in SPRITE_CHARS.iter().enumerate() {
            let p = SPRITE_CHARS_ADDR as usize + i * sprite.len();
            mem[p..p + sprite.len()].copy_from_slice(sprite)
        }
        Self {
            mem: mem,
            v: Regs::new(),
            i: 0,
            pc: ROM_ADDR as u16,
            stack: [0; 0x10],
            sp: 0,
            dt: 0,
            st: 0,
            k: 0,
            fb: [0; SCREEN_WIDTH * SCREEN_HEIGTH / 8],
            rng: rand::rngs::SmallRng::seed_from_u64(seed),
        }
    }
}

impl<R: RngCore> Chip8<R> {
    pub fn load_rom(&mut self, rom: &[u8]) -> Result<(), Error> {
        if rom.len() > MEM_SIZE - ROM_ADDR {
            return Err(Error::RomTooBig(rom.len()));
        }
        self.mem[ROM_ADDR..ROM_ADDR + rom.len()].copy_from_slice(rom);
        // println!("load_rom {:02x}{:02}", self.mem[0x200], self.mem[0x201]);
        Ok(())
    }
    // time is in micro seconds
    pub fn frame(&mut self, time: usize) -> Result<Output, Error> {
        if self.dt != 0 {
            self.dt -= 1;
        }
        let tone = if self.st != 0 {
            self.st -= 1;
            true
        } else {
            false
        };
        let mut rem_time = time as isize;

        while rem_time > 0 {
            if self.pc as usize > MEM_SIZE - 1 {
                return Err(Error::PcOutOfBounds(self.pc));
            }
            let w0 = self.mem[self.pc as usize];
            let w1 = self.mem[self.pc as usize + 1];
            // println!(
            //     "{:04x}: {:02x}{:02x} v0: {} v1: {}",
            //     self.pc, w0, w1, self.v[0], self.v[1]
            // );
            let adv = self.exec(w0, w1)?;
            rem_time = rem_time - adv as isize;
        }
        Ok(Output {
            tone,
            overtime: (rem_time * -1) as usize,
        })
    }

    fn op_cls(&mut self) -> usize {
        for b in self.fb.iter_mut() {
            *b = 0;
        }
        self.pc += 2;
        109
    }
    fn op_call_rca_1802(&mut self, addr: u16) -> usize {
        100
    }
    fn op_ret(&mut self) -> usize {
        self.sp -= 1;
        self.pc = self.stack[self.sp as usize];
        // panic!("DBG");
        105
    }
    fn op_jp(&mut self, addr: u16) -> usize {
        self.pc = addr;
        105
    }
    fn op_call(&mut self, addr: u16) -> usize {
        self.stack[self.sp as usize] = self.pc + 2;
        self.sp += 1;
        self.pc = addr;
        105
    }
    fn op_se(&mut self, a: u8, b: u8) -> usize {
        if a == b {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
        61
    }
    fn op_sne(&mut self, a: u8, b: u8) -> usize {
        if a != b {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
        61
    }
    fn op_ld(&mut self, r: Reg, v: u8) -> usize {
        self.v[r.0] = v;
        self.pc += 2;
        27
    }
    fn op_ld_vx_k(&mut self, r: Reg) -> usize {
        for i in 0..0x10 {
            if 1 << i & self.k != 0 {
                self.v[r.0] = i as u8;
                self.pc += 2;
                break;
            }
        }
        200
    }
    fn op_ld_dt(&mut self, v: u8) -> usize {
        self.dt = v;
        self.pc += 2;
        45
    }
    fn op_ld_st(&mut self, v: u8) -> usize {
        self.st = v;
        self.pc += 2;
        45
    }
    fn op_ld_f(&mut self, v: u8) -> usize {
        self.i = SPRITE_CHARS_ADDR + v as u16 * 5;
        self.pc += 2;
        91
    }
    fn op_ld_b(&mut self, v: u8) -> usize {
        let d2 = v / 100;
        let v = v - d2 * 100;
        let d1 = v / 10;
        let v = v - d1 * 10;
        let d0 = v / 1;
        self.mem[self.i as usize + 0] = d2;
        self.mem[self.i as usize + 1] = d1;
        self.mem[self.i as usize + 2] = d0;
        self.pc += 2;
        927
    }
    fn op_ld_i_vx(&mut self, x: u8) -> usize {
        for i in 0..x + 1 {
            self.mem[self.i as usize + i as usize] = self.v[i];
        }
        self.pc += 2;
        605
    }
    fn op_ld_vx_i(&mut self, x: u8) -> usize {
        for i in 0..x + 1 {
            self.v[i] = self.mem[self.i as usize + i as usize];
        }
        self.pc += 2;
        605
    }
    fn op_add(&mut self, a: Reg, b: u8) -> usize {
        let (res, overflow) = self.v[a.0].overflowing_add(b);
        self.v[a.0] = res;
        self.v[0xf] = if overflow { 1 } else { 0 };
        self.pc += 2;
        45
    }
    fn op_add16(&mut self, b: u8) -> usize {
        self.i += b as u16;
        self.pc += 2;
        86
    }
    fn op_or(&mut self, a: Reg, b: u8) -> usize {
        self.v[a.0] |= b;
        self.pc += 2;
        200
    }
    fn op_and(&mut self, a: Reg, b: u8) -> usize {
        self.v[a.0] &= b;
        self.pc += 2;
        200
    }
    fn op_xor(&mut self, a: Reg, b: u8) -> usize {
        self.v[a.0] ^= b;
        self.pc += 2;
        200
    }
    fn op_sub(&mut self, a: Reg, b: u8) -> usize {
        let (res, overflow) = self.v[a.0].overflowing_sub(b);
        self.v[a.0] = res;
        self.v[0xf] = if overflow { 0 } else { 1 };
        self.pc += 2;
        200
    }
    fn op_subn(&mut self, a: Reg, b: u8) -> usize {
        let (res, overflow) = b.overflowing_sub(self.v[a.0]);
        self.v[a.0] = res;
        self.v[0xf] = if overflow { 0 } else { 1 };
        self.pc += 2;
        200
    }
    fn op_shr(&mut self, a: Reg) -> usize {
        let (res, overflow) = self.v[a.0].overflowing_shr(1);
        self.v[a.0] = res;
        self.v[0xf] = if overflow { 1 } else { 0 };
        self.pc += 2;
        200
    }
    fn op_shl(&mut self, a: Reg) -> usize {
        let (res, overflow) = self.v[a.0].overflowing_shl(1);
        self.v[a.0] = res;
        self.v[0xf] = if overflow { 1 } else { 0 };
        self.pc += 2;
        200
    }
    fn op_ld_i(&mut self, addr: u16) -> usize {
        self.i = addr;
        self.pc += 2;
        55
    }
    fn op_rnd(&mut self, r: Reg, v: u8) -> usize {
        self.v[r.0] = (self.rng.next_u32() as u8) & v;
        self.pc += 2;
        164
    }
    fn op_drw(&mut self, pos_x: u8, pos_y: u8, n: u8) -> usize {
        // println!("DRAW ({}, {})", pos_x, pos_y);
        let pos_x = pos_x % 64;
        let pos_y = pos_y % 32;
        let mut fb = &mut self.fb;
        let shift = pos_x % 8;
        let col_a = pos_x as usize / 8;
        let col_b = (col_a + 1) % (SCREEN_WIDTH / 8);
        let mut collision = 0;
        for i in 0..(n as usize) {
            let byte = self.mem[self.i as usize + i];
            let y = (pos_y as usize + i) % SCREEN_HEIGTH;
            let a = byte >> shift;
            let fb_a = &mut fb[y * SCREEN_WIDTH / 8 + col_a];
            collision |= *fb_a & a;
            *fb_a ^= a;
            // println!("{:08b} {:08b}", byte, *fb_a);
            if shift != 0 {
                let b = byte << (8 - shift);
                let fb_b = &mut fb[y * SCREEN_WIDTH / 8 + col_b];
                collision |= *fb_b & b;
                *fb_b ^= b;
            }
        }
        // DBG
        // for y in 0..SCREEN_HEIGTH {
        //     for x in 0..SCREEN_WIDTH / 8 {
        //         print!("{:08b}", self.fb[y * SCREEN_WIDTH / 8 + x]);
        //     }
        //     println!();
        // }
        self.v[0xF] = if collision != 0 { 1 } else { 0 };
        self.pc += 2;
        22734
    }
    fn op_skp(&mut self, v: u8) -> usize {
        if 1 << v & self.k != 0 {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
        73
    }
    fn op_sknp(&mut self, v: u8) -> usize {
        if 1 << v & self.k == 0 {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
        73
    }
    fn exec(&mut self, w0: u8, w1: u8) -> Result<usize, Error> {
        Ok(match w0 & 0xf0 {
            0x00 => match w1 {
                0xe0 => self.op_cls(),
                0xee => self.op_ret(),
                _ => self.op_call_rca_1802(nnn!(w0, w1)),
            },
            0x10 => self.op_jp(nnn!(w0, w1)),
            0x20 => self.op_call(nnn!(w0, w1)),
            0x30 => self.op_se(self.v[w0 & 0x0f], w1),
            0x40 => self.op_sne(self.v[w0 & 0x0f], w1),
            0x50 => self.op_se(self.v[w0 & 0x0f], self.v[(w1 & 0xf0) >> 4]),
            0x60 => self.op_ld(Reg(w0 & 0x0f), w1),
            0x70 => self.op_add(Reg(w0 & 0x0f), w1),
            0x80 => {
                let a = Reg(w0 & 0x0f);
                let b = self.v[(w1 & 0xf0) >> 4];
                match w1 & 0x0f {
                    0x00 => self.op_ld(Reg(w0 & 0x0f), b),
                    0x01 => self.op_or(a, b),
                    0x02 => self.op_and(a, b),
                    0x03 => self.op_xor(a, b),
                    0x04 => self.op_add(a, b),
                    0x05 => self.op_sub(a, b),
                    0x06 => self.op_shr(a),
                    0x07 => self.op_subn(a, b),
                    0x0E => self.op_shl(a),
                    _ => return Err(Error::InvalidOp(w0, w1)),
                }
            }
            0x90 => match w1 & 0x0f {
                0x00 => self.op_sne(self.v[w0 & 0x0f], self.v[(w1 & 0xf0) >> 4]),
                _ => return Err(Error::InvalidOp(w0, w1)),
            },
            0xA0 => self.op_ld_i(nnn!(w0, w1)),
            0xB0 => self.op_jp(self.v[0] as u16 + nnn!(w0, w1)),
            0xC0 => self.op_rnd(Reg(w0 & 0x0f), w1),
            0xD0 => self.op_drw(self.v[w0 & 0x0f], self.v[(w1 & 0xf0) >> 4], w1 & 0x0f),
            0xE0 => match w1 {
                0x9E => self.op_skp(self.v[w0 & 0x0f]),
                0xA1 => self.op_sknp(self.v[w0 & 0x0f]),
                _ => return Err(Error::InvalidOp(w0, w1)),
            },
            0xF0 => match w1 {
                0x07 => self.op_ld(Reg(w0 & 0x0f), self.dt),
                0x0A => self.op_ld_vx_k(Reg(w0 & 0x0f)),
                0x15 => self.op_ld_dt(self.v[w0 & 0x0f]),
                0x18 => self.op_ld_st(self.v[w0 & 0x0f]),
                0x1E => self.op_add16(self.v[w0 & 0x0f]),
                0x29 => self.op_ld_f(self.v[w0 & 0x0f]),
                0x33 => self.op_ld_b(self.v[w0 & 0x0f]),
                0x55 => self.op_ld_i_vx(w0 & 0x0f),
                0x65 => self.op_ld_vx_i(w0 & 0x0f),
                _ => return Err(Error::InvalidOp(w0, w1)),
            },
            _ => return Err(Error::InvalidOp(w0, w1)),
        })
    }
}
