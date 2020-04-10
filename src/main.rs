use std::ops::{Index, IndexMut};

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
const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGTH: usize = 32;

enum Error {
    InvalidOp(u8, u8),
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

struct chip8 {
    mem: [u8; 0x1000],
    v: Regs, // Register Set
    i: u16,
    pc: u16, // Program Counter
    stack: [u16; 16],
    sp: u8,                // Stack Pointer
    dt: u8,                // Delay Timer
    st: u8,                // Sound TImer
    k: u16,                // Keypad
    fb: [u8; 64 * 32 / 8], // Framebuffer
}

macro_rules! nnn {
    ($w0:expr, $w1:expr) => {
        (($w0 & 0x0f) as u16) << 8 | $w0 as u16
    };
}

impl chip8 {
    fn op_cls(&mut self) {
        for b in self.fb.iter_mut() {
            *b = 0;
        }
        self.pc += 1;
    }
    fn op_call_rca_1802(&mut self, addr: u16) {}
    fn op_ret(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp as usize];
    }
    fn op_jp(&mut self, addr: u16) {
        self.pc = addr;
    }
    fn op_call(&mut self, addr: u16) {
        self.stack[self.sp as usize] = self.pc;
        self.sp += 1;
        self.pc = addr;
    }
    fn op_se(&mut self, a: u8, b: u8) {
        if a == b {
            self.pc += 2;
        } else {
            self.pc += 1;
        }
    }
    fn op_sne(&mut self, a: u8, b: u8) {
        if a != b {
            self.pc += 2;
        } else {
            self.pc += 1;
        }
    }
    fn op_ld(&mut self, r: Reg, v: u8) {
        self.v[r.0] = v;
        self.pc += 1;
    }
    fn op_ld_vx_k(&mut self, v: u8) {
        unimplemented!();
    }
    fn op_ld_dt(&mut self, v: u8) {
        self.dt = v;
        self.pc += 1;
    }
    fn op_ld_st(&mut self, v: u8) {
        self.st = v;
        self.pc += 1;
    }
    fn op_ld_f(&mut self, v: u8) {
        self.i = SPRITE_CHARS_ADDR + v as u16 * 5;
        self.pc += 1;
    }
    fn op_ld_b(&mut self, v: u8) {
        let d2 = (v - 0) / 100;
        let d1 = (v - d2) / 10;
        let d0 = (v - d1) / 1;
        self.mem[self.i as usize + 0] = d2;
        self.mem[self.i as usize + 1] = d1;
        self.mem[self.i as usize + 2] = d0;
        self.pc += 1;
    }
    fn op_ld_i_vx(&mut self, x: u8) {
        for i in 0..x {
            self.mem[self.i as usize + i as usize] = self.v[i];
        }
        self.pc += 1;
    }
    fn op_ld_vx_i(&mut self, x: u8) {
        for i in 0..x {
            self.v[i] = self.mem[self.i as usize + i as usize];
        }
        self.pc += 1;
    }
    fn op_add(&mut self, a: Reg, b: u8) {
        let (res, overflow) = self.v[a.0].overflowing_add(b);
        self.v[a.0] = res;
        self.v[0xf] = if overflow { 1 } else { 0 };
        self.pc += 1;
    }
    fn op_add16(&mut self, b: u8) {
        self.i += b as u16;
        self.pc += 1;
    }
    fn op_or(&mut self, a: Reg, b: u8) {
        self.v[a.0] |= b;
        self.pc += 1;
    }
    fn op_and(&mut self, a: Reg, b: u8) {
        self.v[a.0] &= b;
        self.pc += 1;
    }
    fn op_xor(&mut self, a: Reg, b: u8) {
        self.v[a.0] ^= b;
        self.pc += 1;
    }
    fn op_sub(&mut self, a: Reg, b: u8) {
        let (res, overflow) = self.v[a.0].overflowing_sub(b);
        self.v[a.0] = res;
        self.v[0xf] = if overflow { 0 } else { 1 };
        self.pc += 1;
    }
    fn op_subn(&mut self, a: Reg, b: u8) {
        let (res, overflow) = b.overflowing_sub(self.v[a.0]);
        self.v[a.0] = res;
        self.v[0xf] = if overflow { 0 } else { 1 };
        self.pc += 1;
    }
    fn op_shr(&mut self, a: Reg, b: u8) {
        let (res, overflow) = b.overflowing_shr(self.v[a.0] as u32);
        self.v[a.0] = res;
        self.v[0xf] = if overflow { 0 } else { 1 };
        self.pc += 1;
    }
    fn op_shl(&mut self, a: Reg, b: u8) {
        let (res, overflow) = b.overflowing_shl(self.v[a.0] as u32);
        self.v[a.0] = res;
        self.v[0xf] = if overflow { 0 } else { 1 };
        self.pc += 1;
    }
    fn op_ld_i(&mut self, addr: u16) {
        self.i = addr;
        self.pc += 1;
    }
    fn op_rnd(&mut self, r: Reg, v: u8) {
        unimplemented!();
        self.pc += 1;
    }
    fn op_drw(&mut self, pos_x: u8, pos_y: u8, n: u8) {
        let shift = pos_x % 8;
        let col_a = pos_x as usize / 8;
        let col_b = (col_a + 1) % SCREEN_WIDTH;
        let mut collision = 0;
        for i in 0..(n as usize) {
            let byte = self.mem[self.i as usize + i];
            let y = (pos_y as usize + i) % SCREEN_HEIGTH;
            let a = byte >> shift;
            let fb_a = &mut self.fb[y * SCREEN_WIDTH / 8 + col_a];
            collision |= *fb_a & a;
            *fb_a ^= a;
            let b = byte << (8 - shift);
            let fb_b = &mut self.fb[y * SCREEN_WIDTH / 8 + col_b];
            collision |= *fb_b & b;
            *fb_b ^= b;
        }
        self.v[0xF] = if collision != 0 { 1 } else { 0 };
        self.pc += 1;
    }
    fn op_skp(&mut self, v: u8) {
        if 1 << v & self.k != 0 {
            self.pc += 2;
        } else {
            self.pc += 1;
        }
    }
    fn op_sknp(&mut self, v: u8) {
        if 1 << v & self.k == 0 {
            self.pc += 2;
        } else {
            self.pc += 1;
        }
    }
    fn exec(&mut self, w0: u8, w1: u8) -> Result<(), Error> {
        match w0 & 0xf0 {
            0x00 => match w1 {
                0xe0 => self.op_cls(),
                0xee => self.op_ret(),
                _ => self.op_call_rca_1802(nnn!(w0, w1)),
            },
            0x10 => self.op_jp(nnn!(w0, w1)),
            0x20 => self.op_call(nnn!(w0, w1)),
            0x30 => self.op_se(self.v[w0 & 0x0f], w1),
            0x40 => self.op_sne(self.v[w0 & 0x0f], w1),
            0x50 => self.op_se(self.v[w0 & 0x0f], self.v[w1 & 0xf0 >> 4]),
            0x60 => self.op_ld(Reg(w0 & 0x0f), w1),
            0x70 => self.op_add(Reg(w0 & 0x0f), w1),
            0x80 => {
                let a = Reg(w0 & 0x0f);
                let b = self.v[w1 & 0xf0 >> 4];
                match w1 & 0x0f {
                    0x00 => self.op_ld(Reg(w0 & 0x0f), b),
                    0x01 => self.op_or(a, b),
                    0x02 => self.op_and(a, b),
                    0x03 => self.op_xor(a, b),
                    0x04 => self.op_add(a, b),
                    0x05 => self.op_sub(a, b),
                    0x06 => self.op_shr(a, b),
                    0x07 => self.op_subn(a, b),
                    0x0E => self.op_shl(a, b),
                    _ => return Err(Error::InvalidOp(w0, w1)),
                }
            }
            0x90 => match w1 & 0x0f {
                0x00 => self.op_sne(self.v[w0 & 0x0f], self.v[w1 & 0xf0 >> 4]),
                _ => return Err(Error::InvalidOp(w0, w1)),
            },
            0xA0 => self.op_ld_i(nnn!(w0, w1)),
            0xB0 => self.op_jp(self.v[0] as u16 + nnn!(w0, w1)),
            0xC0 => self.op_rnd(Reg(w0 & 0x0f), w1),
            0xD0 => self.op_drw(self.v[w0 & 0x0f], self.v[w1 & 0xf0 >> 4], w1 & 0x0f),
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
                0x64 => self.op_ld_vx_i(w0 & 0x0f),
                _ => return Err(Error::InvalidOp(w0, w1)),
            },
            _ => return Err(Error::InvalidOp(w0, w1)),
        }
        Ok(())
    }
}

fn main() {
    println!("Hello, world!");
}
