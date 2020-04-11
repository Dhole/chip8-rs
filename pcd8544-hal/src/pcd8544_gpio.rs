// use embedded_hal::blocking::delay::DelayMs;
// use embedded_hal::digital::OutputPin;
//
// use Pcd8544;
//
// pub struct Pcd8544Gpio<CLK, DIN, DC, CS> {
//     clk: CLK,
//     din: DIN,
//     dc: DC,
//     cs: CS,
// }
//
// impl<CLK, DIN, DC, CS> Pcd8544Gpio<CLK, DIN, DC, CS>
// where
//     CLK: OutputPin,
//     DIN: OutputPin,
//     DC: OutputPin,
//     CS: OutputPin,
// {
//     pub fn new(
//         clk: CLK,
//         din: DIN,
//         dc: DC,
//         cs: CS,
//         rst: &mut OutputPin,
//         delay: &mut DelayMs<u8>,
//     ) -> Pcd8544Gpio<CLK, DIN, DC, CS> {
//         rst.set_low();
//         delay.delay_ms(10);
//         rst.set_high();
//
//         let mut pcd = Pcd8544Gpio { clk, din, dc, cs };
//         pcd.init();
//         pcd
//     }
//
//     fn send(&mut self, byte: u8) {
//         for bit in (0..8).rev() {
//             if (byte & (1 << bit)) != 0 {
//                 self.din.set_high();
//             } else {
//                 self.din.set_low();
//             }
//
//             self.clk.set_high();
//             self.clk.set_low();
//         }
//     }
// }
//
// impl<CLK, DIN, DC, CS> Pcd8544 for Pcd8544Gpio<CLK, DIN, DC, CS>
// where
//     CLK: OutputPin,
//     DIN: OutputPin,
//     DC: OutputPin,
//     CS: OutputPin,
// {
//     fn command(&mut self, cmd: u8) {
//         self.dc.set_low();
//         self.cs.set_low();
//         self.send(cmd);
//         self.cs.set_high();
//     }
//
//     fn data(&mut self, data: &[u8]) {
//         self.dc.set_high();
//         self.cs.set_low();
//         for byte in data {
//             self.send(*byte);
//         }
//         self.cs.set_high();
//     }
// }
