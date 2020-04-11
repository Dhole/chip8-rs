use core::convert::Infallible;

use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::spi::Write;
use embedded_hal::digital::v2::OutputPin;

use Pcd8544;

pub struct Pcd8544Spi<SPI, DC, CS> {
    spi: SPI,
    dc: DC,
    cs: CS,
}

impl<SPI, DC, CS> Pcd8544Spi<SPI, DC, CS>
where
    SPI: Write<u8>,
    DC: OutputPin,
    CS: OutputPin,
{
    pub fn new(
        spi: SPI,
        dc: DC,
        cs: CS,
        rst: &mut OutputPin<Error = Infallible>,
        delay: &mut DelayMs<u8>,
    ) -> Pcd8544Spi<SPI, DC, CS> {
        rst.set_low().ok().unwrap();
        delay.delay_ms(10);
        rst.set_high().ok().unwrap();
        delay.delay_ms(10);

        let mut pcd = Pcd8544Spi { spi, dc, cs };
        pcd.init();
        pcd
    }
}

impl<SPI, DC, CS> Pcd8544 for Pcd8544Spi<SPI, DC, CS>
where
    SPI: Write<u8>,
    DC: OutputPin,
    CS: OutputPin,
{
    fn command(&mut self, cmd: u8) {
        self.dc.set_low().ok().unwrap();
        self.cs.set_low().ok().unwrap();
        self.spi.write(&[cmd]);
        self.cs.set_high().ok().unwrap();
    }

    fn data(&mut self, data: &[u8]) {
        self.dc.set_high().ok().unwrap();
        self.cs.set_low().ok().unwrap();
        self.spi.write(data);
        self.cs.set_high().ok().unwrap();
    }

    // fn data_iter<WI>(&mut self, data: WI)
    // where
    //     WI: IntoIterator<Item = u8>,
    // {
    //     self.dc.set_high().ok().unwrap();
    //     self.cs.set_low().ok().unwrap();
    //     for word in data.into_iter() {
    //         self.spi.write(&[word]);
    //     }
    //     self.cs.set_high().ok().unwrap();
    // }
}
