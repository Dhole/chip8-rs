use core::ops::{Index, IndexMut};

use rand::Rng;

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

struct Chip8 {
    mem: [u8; MEM_SIZE],
    v: Regs, // Register Set
    i: u16,
    pc: u16, // Program Counter
    stack: [u16; 0x10],
    sp: u8,                                           // Stack Pointer
    dt: u8,                                           // Delay Timer
    st: u8,                                           // Sound TImer
    pub k: u16,                                       // Keypad
    pub fb_a: [u8; SCREEN_WIDTH * SCREEN_HEIGTH / 8], // Framebuffer A
    pub fb_b: [u8; SCREEN_WIDTH * SCREEN_HEIGTH / 8], // Framebuffer B
    fb_current: u8,
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

impl Chip8 {
    fn new() -> Self {
        Self {
            mem: [0; MEM_SIZE],
            v: Regs::new(),
            i: 0,
            pc: ROM_ADDR as u16,
            stack: [0; 0x10],
            sp: 0,
            dt: 0,
            st: 0,
            k: 0,
            fb_a: [0; SCREEN_WIDTH * SCREEN_HEIGTH / 8],
            fb_b: [0; SCREEN_WIDTH * SCREEN_HEIGTH / 8],
            fb_current: 0,
        }
    }
    fn load_rom(&mut self, rom: &[u8]) -> Result<(), Error> {
        if rom.len() > MEM_SIZE - ROM_ADDR {
            return Err(Error::RomTooBig(rom.len()));
        }
        self.mem[ROM_ADDR..ROM_ADDR + rom.len()].copy_from_slice(rom);
        // println!("load_rom {:02x}{:02}", self.mem[0x200], self.mem[0x201]);
        Ok(())
    }
    // time is in micro seconds
    fn frame(&mut self, time: usize) -> Result<Output, Error> {
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

        if self.fb_current == 0 {
            self.fb_a.copy_from_slice(&self.fb_b);
        } else {
            self.fb_b.copy_from_slice(&self.fb_a);
        }
        while rem_time > 0 {
            if self.pc as usize * 2 > MEM_SIZE - 1 {
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
        self.fb_current = (self.fb_current + 1) % 2;
        Ok(Output {
            tone,
            overtime: (rem_time * -1) as usize,
        })
    }

    fn op_cls(&mut self) -> usize {
        for b in self.fb_a.iter_mut() {
            *b = 0;
        }
        for b in self.fb_b.iter_mut() {
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
        self.v[r.0] = rand::random::<u8>() & v;
        self.pc += 2;
        164
    }
    fn op_drw(&mut self, pos_x: u8, pos_y: u8, n: u8) -> usize {
        // println!("DRAW ({}, {})", pos_x, pos_y);
        let mut fb = if self.fb_current == 0 {
            &mut self.fb_a
        } else {
            &mut self.fb_b
        };
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

use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

use clap::{App, Arg, SubCommand};

use std::fs;
use std::io::{self, Read};
use std::time::Duration;

#[derive(Debug)]
pub enum FrontError {
    Chip8(Error),
    SDL2(String),
    Io(io::Error),
}

impl From<Error> for FrontError {
    fn from(err: Error) -> Self {
        Self::Chip8(err)
    }
}

impl From<io::Error> for FrontError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<String> for FrontError {
    fn from(err: String) -> Self {
        Self::SDL2(err)
    }
}

struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

pub fn main() -> Result<(), FrontError> {
    let app = App::new("Chip8-rs")
        .version("0.0.1")
        .author("Dhole")
        .arg(
            Arg::with_name("scale")
                .short("s")
                .long("scale")
                .value_name("N")
                .help("Sets the scaling factor")
                .takes_value(true)
                .default_value("8")
                .validator(|scale| match scale.parse::<u32>() {
                    Ok(_) => Ok(()),
                    Err(e) => Err(format!("{}", e)),
                }),
        )
        .arg(
            Arg::with_name("path")
                .help("Path to the rom file")
                .index(1)
                .required(true),
        )
        .get_matches();

    let scale = app
        .value_of("scale")
        .map(|s| s.parse::<u32>().unwrap())
        .unwrap();
    let path = app.value_of("path").unwrap();

    let mut rom = Vec::new();
    fs::OpenOptions::new()
        .read(true)
        .open(path)?
        .read_to_end(&mut rom)?;

    // println!("rom {:02x}{:02x}", rom[0], rom[1]);

    let mut chip8 = Chip8::new();
    chip8.load_rom(&rom)?;
    run(scale, &mut chip8)
}

fn run(scale: u32, chip8: &mut Chip8) -> Result<(), FrontError> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let audio_subsystem = sdl_context.audio().unwrap();

    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1), // mono
        samples: None,     // default sample size
    };

    let device = audio_subsystem
        .open_playback(None, &desired_spec, |spec| {
            // initialize the audio callback
            SquareWave {
                phase_inc: 440.0 / spec.freq as f32,
                phase: 0.0,
                volume: 0.25,
            }
        })
        .unwrap();

    let window = video_subsystem
        .window(
            "chip8-rs",
            SCREEN_WIDTH as u32 * scale,
            SCREEN_HEIGTH as u32 * scale,
        )
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump()?;

    'running: loop {
        let mut k = 0 as u16;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    k |= match keycode {
                        Keycode::Num1 => 1 << 0x1,
                        Keycode::Num2 => 1 << 0x2,
                        Keycode::Num3 => 1 << 0x3,
                        Keycode::Num4 => 1 << 0xC,
                        Keycode::Q => 1 << 0x4,
                        Keycode::W => 1 << 0x5,
                        Keycode::E => 1 << 0x6,
                        Keycode::R => 1 << 0xD,
                        Keycode::A => 1 << 0x7,
                        Keycode::S => 1 << 0x8,
                        Keycode::D => 1 << 0x9,
                        Keycode::F => 1 << 0xE,
                        Keycode::Z => 1 << 0xA,
                        Keycode::X => 1 << 0x0,
                        Keycode::C => 1 << 0xB,
                        Keycode::V => 1 << 0xF,
                        _ => 0,
                    };
                }
                _ => {}
            }
        }

        chip8.k = k;
        let out = chip8.frame(16666)?;
        if out.tone {
            device.resume();
        } else {
            device.pause();
        }

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.set_draw_color(Color::RGB(255, 255, 255));

        for y in 0..SCREEN_HEIGTH {
            for x in 0..SCREEN_WIDTH / 8 {
                let byte =
                    chip8.fb_a[y * SCREEN_WIDTH / 8 + x] | chip8.fb_b[y * SCREEN_WIDTH / 8 + x];
                for i in 0..8 {
                    if byte & 1 << (7 - i) != 0 {
                        canvas.fill_rect(Rect::new(
                            (x * 8 + i) as i32 * scale as i32,
                            y as i32 * scale as i32,
                            scale,
                            scale,
                        ))?;
                    }
                }
            }
        }

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        // The rest of the game loop goes here...
    }

    Ok(())
}
