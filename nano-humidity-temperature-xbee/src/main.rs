#![no_std]
#![no_main]

use arduino_hal::{delay_ms, I2c};
use arduino_hal::prelude::*;
use panic_halt as _;
use xbee;
use crate::Command::{NoHoldHumidity, NoHoldTemperature};
use crate::Error::{I2cError, NotSensorValue};

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut led = pins.d13.into_output();
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut i2c = I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        50000,
    );

    loop {
        delay_ms(1000);
        led.toggle();

        let mut xbee_data = [0x00 as u8; 4];
        match read_sensor_value(&mut i2c, NoHoldTemperature) {
            Ok(data) => {
                xbee_data[0] = data[0];
                xbee_data[1] = data[1];
            }
            Err(_) => continue
        }
        match read_sensor_value(&mut i2c, NoHoldHumidity) {
            Ok(data) => {
                xbee_data[2] = data[0];
                xbee_data[3] = data[1];
            }
            Err(_) => continue,
        }

        let packet = xbee::Packet::new(xbee::ApiIdentifier::TxReq,
                                       [0x00, 0x13, 0xA2, 0x00, 0x40, 0x64, 0x03, 0x75],
                                       &xbee_data);
        for byte in packet.iter() {
            serial.write_byte(byte);
        }
        serial.flush();
    }
}

fn read_sensor_value(i2c: &mut I2c, cmd: Command) -> Result<[u8; 2], Error> {
    i2c.write(0x40, &[cmd.value()])?;
    let mut buffer = [0x00, 0x00, 0x00];
    let mut ready = false;
    while !ready {
        delay_ms(10);
        ready = match i2c.read(0x40, &mut buffer) {
            Ok(_) => true,
            Err(_) => false,
        };
    }
    let data = buffer.as_slice().split_at(2).0;
    if !htu21::check(data, &(buffer[2])) {
        return Err(NotSensorValue);
    }
    return Ok([buffer[0], buffer[1]]);
}

fn abs(value: f32) -> f32 {
    return match value {
        v if v < 0.0 => -1.0 * v,
        v => v,
    };
}

enum Command { NoHoldTemperature, NoHoldHumidity }

impl Command {
    fn value(&self) -> u8 {
        match &self {
            NoHoldTemperature => return 0xf3,
            NoHoldHumidity => return 0xf5,
        }
    }
}

#[derive(ufmt::derive::uDebug, Debug, PartialEq)]
enum Error {
    NotSensorValue,
    I2cError(arduino_hal::i2c::Error),
}

impl From<arduino_hal::i2c::Error> for Error {
    fn from(value: arduino_hal::i2c::Error) -> Self {
        return I2cError(value);
    }
}