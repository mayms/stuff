#![no_std]
#![no_main]

use core::mem::transmute;
use panic_halt as _;
use arduino_hal::prelude::*;
use arduino_hal::{delay_ms, I2c, pins, default_serial};
use ufmt::{uwriteln};
use crate::Error::I2cError;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = pins!(dp);

    let mut serial = default_serial!(dp, pins, 57_600);
    let mut i2c = I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        100_000,
    );
    let mut led = pins.d13.into_output();

    loop {
        led.toggle();
        match read(&mut i2c) {
            Ok(data) => {
                let second = data.second();
                let minute = data.minute();
                let hour = data.hour();
                uwriteln!(&mut serial, "{}:{}:{}\r", hour, minute, second).void_unwrap();
            }
            Err(e) => {
                uwriteln!(&mut serial, "error={:?}\r", e).void_unwrap();
            },
        }
        delay_ms(1_000);
    }
}

fn read(i2c: &mut I2c) -> Result<RawData, Error> {
    i2c.write(ADDRESS, &[0x00])?;
    let mut buffer = [0x00; 8];
    i2c.read(ADDRESS, &mut buffer)?;
    let data = unsafe {
        transmute::<[u8;8], RawData>(buffer)
    };
    return Ok(data);
}

const ADDRESS: u8 = 0b0110_1000;

#[repr(C)]
struct RawData {
    second: u8,
    minute: u8,
    hour: u8,
    day: u8,
    date: u8,
    month: u8,
    year: u8,
    control: u8,
}

impl RawData {
    pub fn second(&self) -> u8 {
        ((self.second & 0b0111_0000) >> 4) * 10 + (self.second & 0b0000_1111)
    }

    pub fn minute(&self) -> u8 {
        ((self.minute & 0b0111_0000) >> 4) * 10 + (self.minute & 0b0000_1111)
    }

    pub fn hour(&self) -> u8 {
        ((self.minute & 0b0011_0000) >> 4) * 10 + (self.hour & 0b0000_1111)
    }

    pub fn day(&self) -> u8 {
        self.date & 0b0000_0111
    }

    pub fn date(&self) -> u8 {
        ((self.date & 0b0011_0000) >> 4) * 10 + (self.date & 0b0000_1111)
    }

    pub fn month(&self) -> u8 {
        ((self.month & 0b0001_0000) >> 4) * 10 + (self.month & 0b0000_1111)
    }

    pub fn year(&self) -> u8 {
        ((self.year & 0b1111_0000) >> 4) * 10 + (self.year & 0b0000_1111)
    }

    /// Bit 7: Output Control (OUT).
    /// This bit controls the output level of the SQW/OUT pin when the square-wave
    /// output is disabled.
    /// If SQWE = 0, the logic level on the SQW/OUT pin is 1 if OUT = 1 and is 0 if OUT = 0.
    /// On initial application of power to the device, this bit is typically set to a 0.
    pub fn control_out(&self) -> bool {
        ((self.control & 0b1000_0000) >> 7) > 0
    }

    /// Bit 4: Square-Wave Enable (SQWE).
    /// This bit, when set to logic 1, enables the oscillator output.
    /// The frequency of the square-wave output depends upon the value of the RS0 and RS1 bits.
    /// With the square-wave output set to 1Hz, the clock registers update
    /// on the falling edge of the square wave.
    /// On initial application of power to the device, this bit is typically set to a 0.
    pub fn control_sqwe(&self) -> bool {
        ((self.control & 0b0001_0000) >> 4) > 0
    }

    /// Bits 1 and 0: Rate Select (RS[1:0]).
    /// These bits control the frequency of the square-wave output
    /// when the square- wave output has been enabled.
    /// The following table lists the square-wave frequencies that can be selected with the RS bits.
    /// On initial application of power to the device, these bits are typically set to a 1.
    /// 0 = 1Hz
    /// 1 = 4.096kHz
    /// 2 = 8.192kHz
    /// 3 = 32.768kHz
    pub fn control_rate_select(&self) -> u8 {
        self.control & 0b0000_0011
    }
}

#[derive(ufmt::derive::uDebug, Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
enum Error {
    I2cError(arduino_hal::i2c::Error),
}

impl From<arduino_hal::i2c::Error> for Error {
    fn from(value: arduino_hal::i2c::Error) -> Self {
        return I2cError(value);
    }
}