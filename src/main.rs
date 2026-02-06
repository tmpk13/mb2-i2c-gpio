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

    board.display_pins.col1.set_low().unwrap();

    let mut state = State::LedOff;

    loop {
        state = match state {
            State::LedOff => {
                board.display_pins.row1.set_high().unwrap();
                gpio.gpio_high();
                rprintln!("high");
                State::LedOn
            }
            State::LedOn => {
                board.display_pins.row1.set_low().unwrap();
                gpio.gpio_low();
                rprintln!("low");
                State::LedOff
            }
        };
        timer.delay_ms(500);
    }
}


