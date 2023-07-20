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