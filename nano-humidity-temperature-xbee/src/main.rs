#![no_std]
#![no_main]

use arduino_hal::{delay_ms, I2c};
use arduino_hal::prelude::*;
use panic_halt as _;
use xbee;
use crate::Command::{NoHoldHumidity, NoHoldTemperature};
use crate::Error::{Htu21Error, I2cError, NotSensorValue};

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

    let mut current_temperature: Option<f32> = None;
    let mut current_humidity: Option<f32> = None;
    let mut i = 0 as u8;
    loop {
        delay_ms(1_000);
        led.toggle();
        match read_sensor_values(&mut i2c) {
            Ok((data, temperature, humidity)) => {
                let update_temperature = match current_temperature {
                    None => true,
                    Some(x) => (x - temperature) > 0.1 || (x - temperature) < -0.1,
                };
                let update_humidity = match current_humidity {
                    None => true,
                    Some(x) => (x - humidity) > 0.5 || (x - humidity) < -0.5,
                };
                let update = i > 9 || update_temperature || update_humidity;
                if update {
                    i = 0;
                    current_temperature = Some(temperature);
                    current_humidity = Some(humidity);
                    let packet = xbee::Packet::new(xbee::ApiIdentifier::TxReq,
                                                   [0x00, 0x13, 0xA2, 0x00, 0x40, 0x64, 0x03, 0x75],
                                                   &data);
                    for byte in packet.iter() {
                        serial.write_byte(byte);
                    }
                    serial.flush();
                }
            }
            _ => (),
        }
        i += 1;
    }
}

fn read_sensor_values(i2c: &mut I2c) -> Result<([u8; 4], f32, f32), Error> {
    let mut xbee_data = [0x00 as u8; 4];

    let temperature_data = read_sensor_value(i2c, NoHoldTemperature)?;
    xbee_data[0] = temperature_data[0];
    xbee_data[1] = temperature_data[1];
    let temperature = htu21::parse_temperature(&temperature_data)?;

    let humidity_data = read_sensor_value(i2c, NoHoldHumidity)?;
    xbee_data[2] = humidity_data[0];
    xbee_data[3] = humidity_data[1];
    let humidity = htu21::parse_humidity(&humidity_data)?;

    return Ok((xbee_data, temperature, humidity));
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

enum Command { NoHoldTemperature, NoHoldHumidity }

impl Command {
    fn value(&self) -> u8 {
        match &self {
            NoHoldTemperature => return 0xf3,
            NoHoldHumidity => return 0xf5,
        }
    }
}

#[derive(Debug, PartialEq)]
enum Error {
    NotSensorValue,
    I2cError(arduino_hal::i2c::Error),
    Htu21Error(htu21::Error),
}

impl From<arduino_hal::i2c::Error> for Error {
    fn from(value: arduino_hal::i2c::Error) -> Self {
        return I2cError(value);
    }
}

impl From<htu21::Error> for Error {
    fn from(value: htu21::Error) -> Self {
        return Htu21Error(value);
    }
}