use chip8::{self, Chip8};

use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};

use rand::{self, RngCore};

use clap::{App, Arg};

use std::fs;
use std::io::{self, Read};
use std::time::{Duration, Instant};

#[derive(Debug)]
pub enum FrontError {
    Chip8(chip8::Error),
    SDL2(String),
    Io(io::Error),
}

impl From<chip8::Error> for FrontError {
    fn from(err: chip8::Error) -> Self {
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
        .map(|s| s.parse::<u32>().expect("scale flag can be parsed as u32"))
        .expect("scale argument is defined");
    let path = app.value_of("path").expect("path argument is defined");

    let mut rom = Vec::new();
    fs::OpenOptions::new()
        .read(true)
        .open(path)?
        .read_to_end(&mut rom)?;

    let mut chip8 = Chip8::new(rand::random());
    chip8.load_rom(&rom)?;
    run(scale, &mut chip8)
}

fn run<R: RngCore>(scale: u32, chip8: &mut Chip8<R>) -> Result<(), FrontError> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let audio_subsystem = sdl_context.audio()?;

    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1), // mono
        samples: None,     // default sample size
    };

    let device = audio_subsystem.open_playback(None, &desired_spec, |spec| {
        // initialize the audio callback
        SquareWave {
            phase_inc: 440.0 / spec.freq as f32,
            phase: 0.0,
            volume: 0.25,
        }
    })?;

    let window = video_subsystem
        .window(
            "chip8-rs",
            chip8::SCREEN_WIDTH as u32 * scale,
            chip8::SCREEN_HEIGTH as u32 * scale,
        )
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let texture_creator = canvas.texture_creator();
    let mut tex_display = texture_creator
        .create_texture_streaming(
            PixelFormatEnum::RGB24,
            chip8::SCREEN_WIDTH as u32,
            chip8::SCREEN_HEIGTH as u32,
        )
        .map_err(|e| e.to_string())?;

    let mut event_pump = sdl_context.event_pump()?;

    let frame_duration = Duration::new(0, 1_000_000_000u32 / 60);
    let mut timestamp = Instant::now();
    let mut keypad = 0u16;
    'running: loop {
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
                    keypad |= match keycode {
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
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => {
                    keypad &= !match keycode {
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

        chip8.frame(keypad)?;
        if chip8.tone() {
            device.resume();
        } else {
            device.pause();
        }

        // tex_display.with_lock(None, |buffer: &mut [u8], pitch: usize| {
        //     for y in 0..chip8::SCREEN_HEIGTH {
        //         for x in 0..chip8::SCREEN_WIDTH / 8 {
        //             let byte = chip8.fb()[y * chip8::SCREEN_WIDTH / 8 + x];
        //             for i in 0..8 {
        //                 let offset = y * pitch + (x * 8 + i) * 3;
        //                 let v = if byte & 1 << (7 - i) != 0 { 255 } else { 0 };
        //                 buffer[offset] = v;
        //                 buffer[offset + 1] = v;
        //                 buffer[offset + 2] = v;
        //             }
        //         }
        //     }
        // })?;

        tex_display.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            for y in 0..chip8::SCREEN_HEIGTH {
                for x in 0..chip8::SCREEN_WIDTH / 8 {
                    let byte = chip8.fb()[y * chip8::SCREEN_WIDTH / 8 + x];
                    for i in 0..8 {
                        let offset = y * pitch + (x * 8 + i) * 3;
                        let on = if byte & 1 << (7 - i) != 0 {
                            true
                        } else {
                            false
                        };
                        const FACTOR: u8 = 30;
                        let v = if on {
                            255
                        } else {
                            buffer[offset].saturating_sub(FACTOR)
                        };
                        buffer[offset] = v;
                        buffer[offset + 1] = v;
                        buffer[offset + 2] = v;
                    }
                }
            }
        })?;

        canvas.clear();
        canvas.copy(&tex_display, None, None)?;
        canvas.present();
        let now = Instant::now();
        let sleep_dur = frame_duration
            .checked_sub(now.saturating_duration_since(timestamp))
            .unwrap_or(Duration::new(0, 0));
        ::std::thread::sleep(sleep_dur);
        timestamp = now;
    }

    Ok(())
}
