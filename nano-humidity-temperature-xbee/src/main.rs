#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]
#![feature(abi_avr_interrupt)]

use arduino_hal::{delay_ms, I2c};
use arduino_hal::prelude::*;
use panic_halt as _;
use xbee;
use crate::Command::{NoHoldHumidity, NoHoldTemperature};
use crate::Error::{Htu21Error, I2cError, NotSensorValue};
use avr_device;
use core::arch::asm;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    // let mut led = pins.d13.into_output();
    let mut xbee_sleep = pins.d7.into_output();
    xbee_sleep.set_high();
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut i2c = I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        50000,
    );

    unsafe { // activates wdt interrupt, 1s timeout, enables sleep
        asm!(
        "CLI",
        "WDR",
        "STS 0x60, {0}",    // WDTCSR
        "STS 0x60, {1}",    // WDTCSR
        "out 0x33, {2}",    // SMCR
        "SEI",
        //========WWWW_WWWW
        //========DDDD_DDDD
        //========IIPC_EPPP
        //========FE3E__210
        in(reg) 0b0001_1000 as u8,
        in(reg) 0b0100_0110 as u8, // interrupt, 1s timeout
        //--------XXXX_SSSS
        //--------XXXX_MMME
        //--------XXXX_210-
        in(reg) 0b0000_0101 as u8,  // Power down
        );
    }

    let mut prev_temperature: Option<f32> = None;
    let mut prev_humidity: Option<f32> = None;
    let mut cycles = 0 as u8;
    loop {
        match read_sensor_values(&mut i2c) {
            Ok((data, temperature, humidity)) => {
                let update_temperature = prev_temperature
                    .map_or(true, |prev| exceeds_delta(prev, temperature, 0.1));
                let update_humidity = prev_humidity
                    .map_or(true, |prev| (prev - humidity) > 0.5 || (prev - humidity) < -0.5);
                if cycles > 9 || update_temperature || update_humidity {
                    cycles = 0;
                    prev_temperature = Some(temperature);
                    prev_humidity = Some(humidity);
                    let packet = xbee::Packet::new(xbee::ApiIdentifier::TxReq,
                                                   [0x00, 0x13, 0xA2, 0x00, 0x40, 0x64, 0x03, 0x75],
                                                   &data);
                    xbee_sleep.set_low();
                    delay_ms(200);
                    for byte in packet.iter() {
                        serial.write_byte(byte);
                    }
                    serial.flush();
                    delay_ms(200);
                    xbee_sleep.set_high();
                }
            }
            _ => (),
        }
        cycles += 1;
        delay_ms(10);
        unsafe {
            asm!(
            "sleep"
            );
        }
        delay_ms(10);
    }
}

fn exceeds_delta(a: f32, b: f32, delta: f32) -> bool {
    let x = a - b;
    return x > delta || x < -delta;
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

#[avr_device::interrupt(atmega328p)]
fn WDT() {
    avr_device::interrupt::free(|_| {
        unsafe {
            asm!(
            "wdr"
            );
        }
    });
}