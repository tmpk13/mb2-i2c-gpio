#![no_main]
#![no_std]


// https://github.com/pdx-cs-rust-embedded/blinky-rs/
use cortex_m_rt::entry;
use embedded_hal::{delay::DelayNs, digital::OutputPin, i2c};
use microbit::{
    board::{Board, I2CExternalPins},
    hal::{timer::Timer, twim},
};
use rtt_target::{rtt_init_print, rprintln};                                   
use panic_rtt_target as _;                                                    
use embedded_hal::i2c::{I2c, Error};

// Pin
// 5   :   0000 100
// Hex     0x16


const ADDR: u8 = 0x20;
pub struct GpioExpander<I2C> {
    i2c: I2C,
}
impl<I2C: I2c> GpioExpander<I2C> {
    pub fn new(i2c: I2C) -> Self {
        Self { i2c }
    }

    pub fn gpio_high(&mut self) -> Result<u8, I2C::Error> {
        let mut temp = [0];
        self.i2c.write_read(ADDR, &[0xFF], &mut temp)?;
        Ok(temp[0])
    }
    pub fn gpio_low(&mut self) -> Result<u8, I2C::Error> {
        let mut temp = [0];
        self.i2c.write_read(ADDR, &[0x00], &mut temp)?;
        Ok(temp[0])
    }
    pub fn gpio_write(&mut self, value: u8) -> Result<u8, I2C::Error> {
        let mut temp = [0];
        self.i2c.write_read(ADDR, &[value], &mut temp)?;
        Ok(temp[0])
    }

    pub fn pin_to_hex(&self, pin: u32) -> u32 {
        // 5 -> 16
        2u32.pow(pin-1).clone()
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
    let i2c = twim::Twim::new(board.TWIM0, board.i2c_external.into(), twim::Frequency::K100);
    let mut gpio = GpioExpander::new(i2c);


    let mut state = State::LedOff;


    assert_eq!(gpio.pin_to_hex(5), 0b001_0000);


    loop {
        state = match state {
            State::LedOff => {
                gpio.gpio_high();
                rprintln!("high");
                State::LedOn
            }
            State::LedOn => {
                gpio.gpio_low();
                rprintln!("low");
                State::LedOff
            }
        };
        timer.delay_ms(500);
    }
}


