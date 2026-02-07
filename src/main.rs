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

const P0: u8 = 0x01;
const P1: u8 = 0x02;
const P2: u8 = 0x04;
const P3: u8 = 0x08;
const P4: u8 = 0x10;
const P5: u8 = 0x20;
const P6: u8 = 0x40;
const P7: u8 = 0x80;

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

    pub fn write_pins_delay(
        &mut self,
        pin_values: &[u8],
        delay: u32,
        timer: &mut Timer<microbit::pac::TIMER0>,
    ) -> Result<(), I2C::Error> {
        self.i2c.write(ADDR, &[self.pins_to_hex(pin_values)])?;
        timer.delay_ms(delay);
        Ok(())
    }

    pub fn pin_to_hex(&self, pin: u8) -> u8 {
        2u8.pow((pin as u32) - 1)
    }

    pub fn pins_to_hex(&self, pins: &[u8]) -> u8 {
        // pins: Values for GPIOs 0, 1, 2, 3, 4, 5, 6, 7
        let value = pins.iter().enumerate().fold(0, |sum, (index, pin)| {
            sum + if *pin == 1 {
                self.pin_to_hex(index as u8)
            } else {
                0
            }
        });
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

    /// Delay for specified milliseconds
    macro_rules! d {
        ($delay:expr) => {
            timer.delay_ms($delay)
        };
    }

    /// Handel i2c errors
    macro_rules! i2c_error {
        ($op:expr) => {
            match $op {
                Ok(v) => { v }
                Err(e) => { rprintln!("i2c failure {:?}", e); }
            }
        };
    }

    loop {
        let pins = [P4, P5, P6];

        let _ = pins.iter().enumerate().for_each(|(i, x)| {
            i2c_error!(gpio.write(!x));
            d!(500);
            i2c_error!(gpio.write(!x & !pins[(i + 1) % pins.len()]));
            d!(500)
        });

        // gpio.write(!pin);
        // d!(500);

        // pin += 0x1;

        // gpio.write(!pin-1 & !P5);
        // d!(500);

        // gpio.write(!P4);
        // d!(500);

        // gpio.write(!P4 & !P5);
        // d!(500);

        // gpio.write(!P5);
        // d!(500);

        // gpio.write(!P5 & !P6);
        // d!(500);

        // gpio.write(!P6);
        // d!(500);

        // gpio.write(!P6 & !P4);
        // d!(500);

        // gpio.write_pins_delay(&[1, 1, 1, 1, 1, 0, 1, 1], 100, &mut timer);
        // gpio.write_pins_delay(&[1, 1, 1, 1, 1, 0, 1, 1], 100, &mut timer);
        // gpio.write_pins_delay(&[1, 1, 1, 1, 0, 1, 1, 1], 100, &mut timer);
        // gpio.write_pins_delay(&[1, 1, 1, 1, 1, 1, 0, 1], 100, &mut timer);

        // gpio.off();

        // timer.delay_ms(500);
    }
}
