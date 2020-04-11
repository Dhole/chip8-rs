#![deny(unsafe_code)]
#![deny(warnings)]
#![no_std]

extern crate cortex_m;
extern crate embedded_hal;
extern crate stm32f30x_hal as hal;

use hal::stm32f30x;
use hal::delay::Delay;
use hal::prelude::*;
use hal::gpio::{gpioa, Output, PushPull};

extern crate pcd8544_hal;
use pcd8544_hal::{Pcd8544, Pcd8544Gpio};

fn main() {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = stm32f30x::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);

    let clocks = rcc.cfgr
        .sysclk(64.mhz())
        .pclk1(32.mhz())
        .freeze(&mut flash.acr);

    let mut delay = Delay::new(cp.SYST, clocks);

    let din = gpioa
        .pa7
        .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);
    let clk = gpioa
        .pa5
        .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);

    let dc: gpioa::PA4<Output<PushPull>> = gpioa
        .pa4
        .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper)
        .into();
    let cs: gpioa::PA3<Output<PushPull>> = gpioa
        .pa3
        .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper)
        .into();
    let mut rst: gpioa::PA1<Output<PushPull>> = gpioa
        .pa1
        .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper)
        .into();

    let mut pcd8544 = Pcd8544Gpio::new(clk, din, dc, cs, &mut rst, &mut delay);

    pcd8544.print("Hello world!");

    loop {
        delay.delay_ms(200u16);
        pcd8544.command(0x0D);
        delay.delay_ms(200u16);
        pcd8544.command(0x0C);
    }
}
