#![deny(unsafe_code)]
#![deny(warnings)]
#![no_std]

extern crate embedded_hal;
extern crate tm4c123x_hal;

use embedded_hal::blocking::delay::DelayMs;

use tm4c123x_hal::delay::Delay;
use tm4c123x_hal::gpio::GpioExt;
use tm4c123x_hal::sysctl::{self, SysctlExt};

extern crate pcd8544_hal;
use pcd8544_hal::{Pcd8544, Pcd8544Gpio};

fn main() {
    let p = tm4c123x_hal::Peripherals::take().unwrap();
    let cp = tm4c123x_hal::CorePeripherals::take().unwrap();

    let mut sc = p.SYSCTL.constrain();
    sc.clock_setup.oscillator = sysctl::Oscillator::Main(
        sysctl::CrystalFrequency::_16mhz,
        sysctl::SystemClock::UsePll(sysctl::PllOutputFrequency::_80_00mhz),
    );

    let clocks = sc.clock_setup.freeze();
    let mut delay = Delay::new(cp.SYST, &clocks);

    let gpioa = p.GPIO_PORTA.split(&sc.power_control);

    let din = gpioa
        .pa5
        .into_push_pull_output();
    let clk = gpioa
        .pa2
        .into_push_pull_output();

    let dc = gpioa
        .pa6
        .into_push_pull_output();
    let cs = gpioa
        .pa3
        .into_push_pull_output();
    let mut rst = gpioa
        .pa7
        .into_push_pull_output();

    let mut pcd8544 = Pcd8544Gpio::new(clk, din, dc, cs, &mut rst, &mut delay);

    loop {
        for i in 0..6 {
            pcd8544.set_position(0, i);
            pcd8544.print("Hello world!");
            delay.delay_ms(500u16);
            pcd8544.clear();
        }
    }
}
