#![no_main]
#![no_std]

use cortex_m_rt::entry;
use embedded_hal::delay::DelayNs;
use embedded_hal::i2c::I2c;
use microbit::{
    board::Board,
    hal::{timer::Timer, twim},
    pac::TIMER0,
};
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};


// Pin definitions
#[allow(unused)]
mod pins {
    pub const P0: u8 = 0x01;
    pub const P1: u8 = 0x02;
    pub const P2: u8 = 0x04;
    pub const P3: u8 = 0x08;
    pub const P4: u8 = 0x10;
    pub const P5: u8 = 0x20;
    pub const P6: u8 = 0x40;
    pub const P7: u8 = 0x80;
}

const ADDR: u8 = 0x20;
pub struct GpioExpander<I2C> {
    i2c: I2C,
}
impl<I2C: I2c> GpioExpander<I2C> {
    pub fn new(i2c: I2C) -> Self {
        Self { i2c }
    }

    /// Set all low
    pub fn on(&mut self) -> Result<(), I2C::Error> {
        self.i2c.write(ADDR, &[0x00])?;
        Ok(())
    }
    /// Set all high
    pub fn off(&mut self) -> Result<(), I2C::Error> {
        self.i2c.write(ADDR, &[0xFF])?;
        Ok(())
    }
    /// Write 8 bits to i2c device
    pub fn write(&mut self, value: u8) -> Result<(), I2C::Error> {
        self.i2c.write(ADDR, &[value])?;
        Ok(())
    }

    /// Write High/Low bits to i2c device from list of bits for pins [1, ..., 1]
    pub fn write_pins(&mut self, pin_values: &[u8]) -> Result<(), I2C::Error> {
        self.i2c.write(ADDR, &[self.pins_to_hex(pin_values)])?;
        Ok(())
    }

    /// Convert pin number to u8 bits
    pub fn pin_to_hex(&self, pin: u8) -> u8 {
        2u8.pow(pin as u32)
    }

    /// Join list of bits [1, ..., 1] -> 0b1...1. Input: P0...7, Output: P7...0
    pub fn pins_to_hex(&self, pins: &[u8]) -> u8 {
        // pins: Values for GPIOs 0, 1, 2, 3, 4, 5, 6, 7
        let value = pins.iter().enumerate().fold(0, |sum, (index, pin)| {
            sum + if *pin == 1 {
                self.pin_to_hex(index as u8)
            } else {
                0
            }
        });
        value.to_be()
    }
}

#[entry]
fn init() -> ! {
    rtt_init_print!();

    let board = Board::take().unwrap();
    let mut timer: Timer<microbit::pac::TIMER0> = Timer::new(board.TIMER0);

    // Setup I2C two wire interface
    let i2c = twim::Twim::new(
        board.TWIM0,
        board.i2c_external.into(),
        twim::Frequency::K100,
    );

    // Setup gpio from expander
    let mut expander_gpio = GpioExpander::new(i2c);

    // Cycle though the colors on 3 pin rgb led (common +)
    fn cycle<T>(
        timer: &mut Timer<TIMER0>,
        pins: &[u8],
        gpio: &mut GpioExpander<T>,
    ) -> Result<(), T::Error>
    where
        T: I2c,
    {
        for (i, pin) in pins.iter().enumerate() {
            // Write pins, inversed pulling low the specified pin
            gpio.write(!pin)?;
            timer.delay_ms(500);
            // Write to current pin and next pin for color blend
            gpio.write(!pin & !pins[(i + 1) % pins.len()])?;
            timer.delay_ms(500);
        }
        Ok(())
    }

    loop {
        use pins::*;

        // Cycle through colors
        match cycle(&mut timer, &[P4, P5, P6], &mut expander_gpio) {
            Ok(()) => {}
            Err(e) => {
                rprintln!("I2C failed {:?}", e)
            }
        }

        // gpio.write(!P6);
        // gpio.write(!P6 & !P4);
        // gpio.off();
        // timer.delay_ms(500);
    }
}
