#![deny(unsafe_code)]
#![no_std]
#![no_main]

use core::fmt::Write;

use panic_halt as _;

use nb::block;

use cortex_m::peripheral::DWT;
use cortex_m_rt::entry;
use embedded_hal::digital::v2::{InputPin, OutputPin};

use stm32f1xx_hal as hal;
use stm32f1xx_hal::prelude::*;
use stm32f1xx_hal::stm32;

use hal::delay::Delay;
use hal::spi::{self, Spi};
use hal::timer::{Tim2NoRemap, Tim3NoRemap, Timer};

use arrayvec::{ArrayString, ArrayVec};

use pcd8544_hal::{self, Pcd8544, Pcd8544Spi};

use chip8::{self, Chip8};
static ROM_GUESS: &'static [u8] = include_bytes!("../../games/GUESS");
static ROM_VBRIX: &'static [u8] = include_bytes!("../../games/VBRIX");
static ROM_SYZYGY: &'static [u8] = include_bytes!("../../games/SYZYGY");
static ROM_KALEID: &'static [u8] = include_bytes!("../../games/KALEID");
static ROM_AIRPLANE_CH8: &'static [u8] = include_bytes!("../../games/Airplane.ch8");
static ROM_CAVE_CH8: &'static [u8] = include_bytes!("../../games/Cave.ch8");
static ROM_TEST_OPCODE_CH8: &'static [u8] = include_bytes!("../../games/test_opcode.ch8");
static ROM_SHOOTINGSTARS_CH8: &'static [u8] = include_bytes!("../../games/ShootingStars.ch8");
static ROM_BOWLING_CH8: &'static [u8] = include_bytes!("../../games/Bowling.ch8");
static ROM_TICTAC: &'static [u8] = include_bytes!("../../games/TICTAC");
static ROM_CONNECT4: &'static [u8] = include_bytes!("../../games/CONNECT4");
static ROM_SUBMARINE_CH8: &'static [u8] = include_bytes!("../../games/Submarine.ch8");
static ROM_TRON_CH8: &'static [u8] = include_bytes!("../../games/Tron.ch8");
static ROM_UFO: &'static [u8] = include_bytes!("../../games/UFO");
static ROM_PADDLES_CH8: &'static [u8] = include_bytes!("../../games/Paddles.ch8");
static ROM_HILO_CH8: &'static [u8] = include_bytes!("../../games/HiLo.ch8");
static ROM_BLITZ: &'static [u8] = include_bytes!("../../games/BLITZ");
static ROM_SOCCER_CH8: &'static [u8] = include_bytes!("../../games/Soccer.ch8");
static ROM_BRIX: &'static [u8] = include_bytes!("../../games/BRIX");
static ROM_BLINKY: &'static [u8] = include_bytes!("../../games/BLINKY");
static ROM_HIDDEN: &'static [u8] = include_bytes!("../../games/HIDDEN");
static ROM_PONG2: &'static [u8] = include_bytes!("../../games/PONG2");
static ROM_TETRIS: &'static [u8] = include_bytes!("../../games/TETRIS");
static ROM_MAZE: &'static [u8] = include_bytes!("../../games/MAZE");
static ROM_SPACEINTERCEPT_CH8: &'static [u8] = include_bytes!("../../games/SpaceIntercept.ch8");
static ROM_BLINKY_CH8: &'static [u8] = include_bytes!("../../games/Blinky.ch8");
static ROM_PUZZLE: &'static [u8] = include_bytes!("../../games/PUZZLE");
static ROM_FILTER_CH8: &'static [u8] = include_bytes!("../../games/Filter.ch8");
static ROM_MERLIN: &'static [u8] = include_bytes!("../../games/MERLIN");
static ROM_MISSILE: &'static [u8] = include_bytes!("../../games/MISSILE");
static ROM_15PUZZLE: &'static [u8] = include_bytes!("../../games/15PUZZLE");
static ROM_LANDING_CH8: &'static [u8] = include_bytes!("../../games/Landing.ch8");
static ROM_PONG: &'static [u8] = include_bytes!("../../games/PONG");
static ROM_INVADERS: &'static [u8] = include_bytes!("../../games/INVADERS");
static ROM_WIPEOFF: &'static [u8] = include_bytes!("../../games/WIPEOFF");
static ROM_ASTRODODGE_CH8: &'static [u8] = include_bytes!("../../games/AstroDodge.ch8");
static ROM_SPACEFLIGHT_CH8: &'static [u8] = include_bytes!("../../games/SpaceFlight.ch8");
static ROM_WORM_CH8: &'static [u8] = include_bytes!("../../games/Worm.ch8");
static ROM_VERS: &'static [u8] = include_bytes!("../../games/VERS");
static ROM_HIDDEN_CH8: &'static [u8] = include_bytes!("../../games/Hidden.ch8");
static ROM_LUNARLANDER_CH8: &'static [u8] = include_bytes!("../../games/LunarLander.ch8");
static ROM_SQUASH_CH8: &'static [u8] = include_bytes!("../../games/Squash.ch8");
static ROM_TANK: &'static [u8] = include_bytes!("../../games/TANK");

const SYSCLK: u32 = 72_000_000;

fn key_pressed<O: OutputPin, I: InputPin>(r: &mut [O; 4], c: &mut [I; 4]) -> u16 {
    for pin in r.iter_mut() {
        pin.set_low().ok().unwrap();
    }
    for y in 0..4 {
        r[y].set_high().ok().unwrap();
        let is_high = [
            c[0].is_high().ok().unwrap(),
            c[1].is_high().ok().unwrap(),
            c[2].is_high().ok().unwrap(),
            c[3].is_high().ok().unwrap(),
        ];
        for x in (0..4).rev() {
            if is_high[x] {
                return 1 << (y * 4 + x);
            }
        }
        r[y].set_low().ok().unwrap();
    }
    0
}

const KEY_0: u16 = 1 << 0xD;
const KEY_1: u16 = 1 << 0x0;
const KEY_2: u16 = 1 << 0x1;
const KEY_3: u16 = 1 << 0x2;
const KEY_4: u16 = 1 << 0x4;
const KEY_5: u16 = 1 << 0x5;
const KEY_6: u16 = 1 << 0x6;
const KEY_7: u16 = 1 << 0x8;
const KEY_8: u16 = 1 << 0x9;
const KEY_9: u16 = 1 << 0xA;
const KEY_A: u16 = 1 << 0xC;
const KEY_B: u16 = 1 << 0xE;
const KEY_C: u16 = 1 << 0x3;
const KEY_D: u16 = 1 << 0x7;
const KEY_E: u16 = 1 << 0xB;
const KEY_F: u16 = 1 << 0xF;

fn key_map(k: u16) -> u16 {
    match k {
        KEY_0 => 1 << 0x0,
        KEY_1 => 1 << 0x1,
        KEY_2 => 1 << 0x2,
        KEY_3 => 1 << 0x3,
        KEY_4 => 1 << 0x4,
        KEY_5 => 1 << 0x5,
        KEY_6 => 1 << 0x6,
        KEY_7 => 1 << 0x7,
        KEY_8 => 1 << 0x8,
        KEY_9 => 1 << 0x9,
        KEY_A => 1 << 0xA,
        KEY_B => 1 << 0xB,
        KEY_C => 1 << 0xC,
        KEY_D => 1 << 0xD,
        KEY_E => 1 << 0xE,
        KEY_F => 1 << 0xF,
        0 => 0,
        _ => unreachable!(),
    }
}

#[entry]
fn main() -> ! {
    // Get access to the core peripherals from the cortex-m crate
    let cp = cortex_m::Peripherals::take().unwrap();
    // Get access to the device specific peripherals from the peripheral access crate
    let dp = stm32::Peripherals::take().unwrap();

    // Take ownership over the raw flash and rcc devices and convert them into the corresponding
    // HAL structs
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);

    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(stm32f1xx_hal::time::Hertz(SYSCLK))
        .pclk1(36.mhz())
        .freeze(&mut flash.acr);

    // Acquire the GPIOC peripheral
    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);
    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);
    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);

    // Configure gpio C pin 13 as a push-pull output. The `crh` register is passed to the function
    // in order to configure the port. For pins 0-7, crl should be passed instead.
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    let mut delay = Delay::new(cp.SYST, clocks);

    // setup SPI
    let sck = gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl);
    let miso = gpioa.pa6.into_floating_input(&mut gpioa.crl);
    let mosi = gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl);
    // let sck = gpiob.pb13.into_alternate_push_pull(&mut gpiob.crh);
    // let miso = gpiob.pb14.into_floating_input(&mut gpiob.crh);
    // let mosi = gpiob.pb15.into_alternate_push_pull(&mut gpiob.crh);
    let spi_mode = spi::Mode {
        phase: spi::Phase::CaptureOnFirstTransition,
        polarity: spi::Polarity::IdleLow,
    };

    let spi = Spi::spi1(
        dp.SPI1,
        (sck, miso, mosi),
        &mut afio.mapr,
        spi_mode,
        // 500.khz(),
        4.mhz(),
        clocks,
        &mut rcc.apb2,
    );
    // let spi = Spi::spi2(
    //     dp.SPI2,
    //     (sck, miso, mosi),
    //     spi_mode,
    //     // 500.khz(),
    //     4.mhz(),
    //     clocks,
    //     &mut rcc.apb1,
    // );

    // other pins for PCD8544
    let mut dc = gpioa.pa4.into_push_pull_output(&mut gpioa.crl);
    let mut cs = gpioa.pa3.into_push_pull_output(&mut gpioa.crl);
    let mut rst = gpioa.pa1.into_push_pull_output(&mut gpioa.crl);

    let mut pcd8544 = Pcd8544Spi::new(spi, dc, cs, &mut rst, &mut delay);

    let mut keypad_r = [
        gpiob.pb12.into_push_pull_output(&mut gpiob.crh).downgrade(),
        gpiob.pb13.into_push_pull_output(&mut gpiob.crh).downgrade(),
        gpiob.pb14.into_push_pull_output(&mut gpiob.crh).downgrade(),
        gpiob.pb15.into_push_pull_output(&mut gpiob.crh).downgrade(),
    ];
    let mut keypad_c = [
        gpioa.pa8.into_pull_down_input(&mut gpioa.crh).downgrade(),
        gpioa.pa9.into_pull_down_input(&mut gpioa.crh).downgrade(),
        gpioa.pa10.into_pull_down_input(&mut gpioa.crh).downgrade(),
        gpioa.pa11.into_pull_down_input(&mut gpioa.crh).downgrade(),
    ];

    // TIM2 PWM
    let c1 = gpioa.pa0.into_alternate_push_pull(&mut gpioa.crl);
    let mut pwm_tone = Timer::tim2(dp.TIM2, &clocks, &mut rcc.apb1).pwm::<Tim2NoRemap, _, _, _>(
        (c1),
        &mut afio.mapr,
        440.hz(),
    );
    let max = pwm_tone.get_max_duty();
    pwm_tone.set_duty(max / 2);

    // TIM3 PWM
    let c4 = gpiob.pb1.into_alternate_push_pull(&mut gpiob.crl);
    let mut pwm_light = Timer::tim3(dp.TIM3, &clocks, &mut rcc.apb1).pwm::<Tim3NoRemap, _, _, _>(
        (c4),
        &mut afio.mapr,
        8.khz(),
    );
    let max_light = pwm_light.get_max_duty();
    const light_levels: u16 = 4;
    pwm_light.set_duty(max_light / light_levels * light_levels);
    pwm_light.enable();

    let syst = delay.free();
    let mut timer = Timer::syst(syst, &clocks).start_count_down(60.hz());

    led.set_high();

    let ROMS = [
        ("GUESS", ROM_GUESS),
        ("VBRIX", ROM_VBRIX),
        ("SYZYGY", ROM_SYZYGY),
        ("KALEID", ROM_KALEID),
        ("AIRPLANE_CH8", ROM_AIRPLANE_CH8),
        ("CAVE_CH8", ROM_CAVE_CH8),
        ("TEST_OPCODE_CH8", ROM_TEST_OPCODE_CH8),
        ("SHOOTINGSTARS_CH8", ROM_SHOOTINGSTARS_CH8),
        ("BOWLING_CH8", ROM_BOWLING_CH8),
        ("TICTAC", ROM_TICTAC),
        ("CONNECT4", ROM_CONNECT4),
        ("SUBMARINE_CH8", ROM_SUBMARINE_CH8),
        ("TRON_CH8", ROM_TRON_CH8),
        ("UFO", ROM_UFO),
        ("PADDLES_CH8", ROM_PADDLES_CH8),
        ("HILO_CH8", ROM_HILO_CH8),
        ("BLITZ", ROM_BLITZ),
        ("SOCCER_CH8", ROM_SOCCER_CH8),
        ("BRIX", ROM_BRIX),
        ("BLINKY", ROM_BLINKY),
        ("HIDDEN", ROM_HIDDEN),
        ("PONG2", ROM_PONG2),
        ("TETRIS", ROM_TETRIS),
        ("MAZE", ROM_MAZE),
        ("SPACEINTERCEPT_CH8", ROM_SPACEINTERCEPT_CH8),
        ("BLINKY_CH8", ROM_BLINKY_CH8),
        ("PUZZLE", ROM_PUZZLE),
        ("FILTER_CH8", ROM_FILTER_CH8),
        ("MERLIN", ROM_MERLIN),
        ("MISSILE", ROM_MISSILE),
        ("15PUZZLE", ROM_15PUZZLE),
        ("LANDING_CH8", ROM_LANDING_CH8),
        ("PONG", ROM_PONG),
        ("INVADERS", ROM_INVADERS),
        ("WIPEOFF", ROM_WIPEOFF),
        ("ASTRODODGE_CH8", ROM_ASTRODODGE_CH8),
        ("SPACEFLIGHT_CH8", ROM_SPACEFLIGHT_CH8),
        ("WORM_CH8", ROM_WORM_CH8),
        ("VERS", ROM_VERS),
        ("HIDDEN_CH8", ROM_HIDDEN_CH8),
        ("LUNARLANDER_CH8", ROM_LUNARLANDER_CH8),
        ("SQUASH_CH8", ROM_SQUASH_CH8),
        ("TANK", ROM_TANK),
    ];

    // menu loop
    let mut msg = ArrayString::<[u8; 64]>::new();
    let mut light: i16 = 0;
    let mut rom_n: i16 = 0;
    let mut key_prev = 0;
    loop {
        block!(timer.wait()).unwrap();
        let key = key_pressed(&mut keypad_r, &mut keypad_c);
        if key_prev == 0 {
            if key == KEY_C {
                light = core::cmp::min(light_levels as i16, light + 1);
            } else if key == KEY_D {
                light = core::cmp::max(0, light - 1);
            } else if key == KEY_8 {
                rom_n = (rom_n + 1) % ROMS.len() as i16;
            } else if key == KEY_2 {
                rom_n = rom_n - 1;
                if rom_n < 0 {
                    rom_n = ROMS.len() as i16 - 1;
                }
            } else if key == KEY_5 {
                break;
            }
        }

        pwm_light.set_duty(max_light / light_levels * (light_levels - light as u16));

        pcd8544.clear();
        msg.clear();
        write!(msg, "light: {}", light);
        pcd8544.set_position(0, 0);
        pcd8544.print(&msg);
        msg.clear();
        write!(msg, "{:02}: {}", rom_n, ROMS[rom_n as usize].0);
        pcd8544.set_position(0, 2);
        pcd8544.print(&msg);

        led.toggle();
        key_prev = key;
    }

    let mut chip8 = Chip8::new(DWT::get_cycle_count() as u64);
    chip8.load_rom(ROMS[rom_n as usize].1).unwrap();

    // chip8 loop
    const frame_us: usize = 1_000_000 / 60;

    const DISP_WIDTH: usize = 84;
    const DISP_HEIGHT: usize = 48;
    let mut disp_fb = [0; DISP_WIDTH * DISP_HEIGHT / 8];
    // let mut overtime: usize = 0;
    loop {
        block!(timer.wait()).unwrap();
        let key = key_map(key_pressed(&mut keypad_r, &mut keypad_c));
        chip8.k = key;
        let out = chip8.frame(frame_us).unwrap();
        if out.tone {
            pwm_tone.enable();
        } else {
            pwm_tone.disable();
        }
        // DBG
        // disp_fb[0] = 0xff;
        // disp_fb[2] = 0xff;
        for b in disp_fb.iter_mut() {
            *b = 0x00;
        }
        for y in 0..chip8::SCREEN_HEIGTH {
            for x in 0..chip8::SCREEN_WIDTH / 8 {
                let byte = chip8.fb[y * chip8::SCREEN_WIDTH / 8 + x];
                for i in 0..8 {
                    let b = (byte & (1 << i)) >> i << (y % 8);
                    disp_fb[(10 + x * 8 + 7 - i) * DISP_HEIGHT / 8 + y / 8] |= b;
                }
                // disp_fb[y * DISP_WIDTH / 8 + 1 + x] = byte;
            }
        }
        pcd8544.draw_buffer(&disp_fb);
        led.toggle();
    }
}
