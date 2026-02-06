#![no_main]
#![no_std]

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

// 0x03
const H: u8 = 0x42;

enum Pins {
    P0 = 0x01,
    P1 = 0x02,
    P2 = 0x04,
    P3 = 0x08,
    P4 = 0x10,
    P5 = 0x20,
    P6 = 0x40,
    P7 = 0x80,
}

const ADDR: u8 = 0x20;
pub struct GpioExpander<I2C> {
    i2c: I2C,
}
impl<I2C: I2c> GpioExpander<I2C> {
    pub fn new(i2c: I2C) -> Self {
        Self { i2c }
    }

    pub fn low(&mut self) -> Result<(), I2C::Error> {
        self.i2c.write(ADDR, &[0x00])?;
        Ok(())
    }
    pub fn high(&mut self) -> Result<(), I2C::Error> {
        self.i2c.write(ADDR, &[0xFF])?;
        Ok(())
    }
    pub fn write(&mut self, value: u8) -> Result<(), I2C::Error> {
        self.i2c.write(ADDR, &[value])?;
        Ok(())
    }

    pub fn pin_to_hex(&self, pin: u8) -> u8 {
        2u8.pow((pin as u32) - 1)
    }

    pub fn pins_to_hex(&self, pins: &[u8]) -> u8 {
        let mut value: u8 = 0x00;
        // Add up pins
        // for pin in 0usize..8usize {
        //     value += 2u8.pow((pins[pin] as u32) - 1);
        // }
        value = pins.iter().fold(
            0, 
            |sum, pin| sum + 2u8.pow((*pin-1) as u32)
        );
        value
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
    let mut timer = Timer::new(board.TIMER0);

    //board.i2c_external
    let i2c = twim::Twim::new(
        board.TWIM0,
        board.i2c_external.into(),
        twim::Frequency::K100,
    );
    let mut gpio = GpioExpander::new(i2c);

    let mut state = State::LedOff;

    assert_eq!(gpio.pin_to_hex(5), 0b0001_0000);

    loop {
        state = match state {
            State::LedOff => {
                gpio.high();
                rprintln!("high");
                State::LedOn
            }
            State::LedOn => {
                gpio.low();
                rprintln!("low");
                State::LedOff
            }
        };
        timer.delay_ms(500);
    }
}
