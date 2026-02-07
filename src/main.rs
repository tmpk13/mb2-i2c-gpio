#![no_main]
#![no_std]

use core::fmt::Binary;

// https://github.com/pdx-cs-rust-embedded/blinky-rs/
use cortex_m_rt::entry;
use embedded_hal::i2c::{Error, I2c};
use embedded_hal::{delay::DelayNs, digital::OutputPin, i2c};
use microbit::{
    board::{Board, I2CExternalPins},
    hal::{timer::Timer, twim},
};
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};


const P0:u8 = 0x01;
const P1:u8 = 0x02;
const P2:u8 = 0x04;
const P3:u8 = 0x08;
const P4:u8 = 0x10;
const P5:u8 = 0x20;
const P6:u8 = 0x40;
const P7:u8 = 0x80;


const ADDR: u8 = 0x20;
pub struct GpioExpander<I2C> {
    i2c: I2C,
}
impl<I2C: I2c> GpioExpander<I2C> {
    pub fn new(i2c: I2C) -> Self {
        Self { i2c }
    }

    pub fn on(&mut self) -> Result<(), I2C::Error> {
        self.i2c.write(ADDR, &[0x00])?;
        Ok(())
    }
    pub fn off(&mut self) -> Result<(), I2C::Error> {
        self.i2c.write(ADDR, &[0xFF])?;
        Ok(())
    }
    pub fn write(&mut self, value: u8) -> Result<(), I2C::Error> {
        self.i2c.write(ADDR, &[value])?;
        Ok(())
    }

    pub fn write_pins(&mut self, pin_values: &[u8]) -> Result<(), I2C::Error> {
        self.i2c.write(ADDR, &[self.pins_to_hex(pin_values)])?;
        Ok(())
    }

    pub fn write_pins_delay(&mut self, pin_values: &[u8], delay: u32, timer: &mut Timer<microbit::pac::TIMER0>) -> Result<(), I2C::Error> {
        self.i2c.write(ADDR, &[self.pins_to_hex(pin_values)])?;
        timer.delay_ms(delay);
        Ok(())
    }

    pub fn pin_to_hex(&self, pin: u8) -> u8 {
        2u8.pow((pin as u32) - 1)
    }

    pub fn pins_to_hex(&self, pins: &[u8]) -> u8 {
        // pins: Values for GPIOs 0, 1, 2, 3, 4, 5, 6, 7
        let value = pins.iter().enumerate().fold(0, |sum, (index, pin)| sum +
            if *pin == 1 {
                self.pin_to_hex(index as u8)
            } else {
                0
            }
        );
        rprintln!("Pin register {:#} {:08b}", value, value.to_be());
        value.to_be()
    }
}

enum State {
    LedOn,
    LedOff,
}

#[entry]
fn init() -> ! {
    rtt_init_print!();
    let mut board = Board::take().unwrap();
    let mut timer: Timer<microbit::pac::TIMER0> = Timer::new(board.TIMER0);

    //board.i2c_external
    let i2c = twim::Twim::new(
        board.TWIM0,
        board.i2c_external.into(),
        twim::Frequency::K100,
    );
    let mut gpio = GpioExpander::new(i2c);

    let mut state = State::LedOff;
    
    #[cfg(test)]
    {
        assert_eq!(gpio.pin_to_hex(5), 0b0001_0000);
        assert_eq!(gpio.pins_to_hex(&[0, 1, 1, 1, 0, 0, 0, 0]), 0b0000_1110);
    }

    

    loop {
        
        rprintln!("{:b} {:b} {:b}", P4, P5, P6);

        gpio.write(!P4);
        timer.delay_ms(500);

        gpio.write(!P4 ^ !P5);
        timer.delay_ms(500);
        
        gpio.write(!P5);
        timer.delay_ms(500);

        gpio.write(!P6);
        timer.delay_ms(500);

        // gpio.write_pins_delay(&[1, 1, 1, 1, 1, 0, 1, 1], 100, &mut timer);
        // gpio.write_pins_delay(&[1, 1, 1, 1, 1, 0, 1, 1], 100, &mut timer);
        // gpio.write_pins_delay(&[1, 1, 1, 1, 0, 1, 1, 1], 100, &mut timer);
        // gpio.write_pins_delay(&[1, 1, 1, 1, 1, 1, 0, 1], 100, &mut timer);

        gpio.off();
        
        timer.delay_ms(500);
    }
}
