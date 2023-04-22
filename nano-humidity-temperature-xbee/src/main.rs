#![no_std]
#![no_main]

use arduino_hal::{DefaultClock, delay_ms};
use arduino_hal::hal::Atmega;
use arduino_hal::hal::port::{PC4, PC5};
use arduino_hal::prelude::*;
use arduino_hal::i2c::{Direction, Error};
use arduino_hal::pac::TWI;
use arduino_hal::port::mode::Input;
use panic_halt as _;
use xbee;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut led = pins.d13.into_output();
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        50000,
    );

    loop {
        led.toggle();

        let mut xbee_data = [0x00 as u8; 4];

        let result = i2c.write(0x40, &[0xf3]);
        match result {
            Ok(_) => {
                // ufmt::uwriteln!(&mut serial, "Ok no hold trigger temp reading\r").void_unwrap();
                let mut buffer = [0x00, 0x00, 0x00];
                let mut ready = false;
                while !ready {
                    delay_ms(10);
                    let read_result = i2c.read(0x40, &mut buffer);
                    ready = match read_result {
                        Ok(_) => true,
                        Err(_) => false,
                    };
                    // ufmt::uwriteln!(&mut serial, "needs delay\r").void_unwrap();
                }
                let data = buffer.as_slice().split_at(2).0;
                let check1 = htu21::check(data, &(buffer[2]));
                // ufmt::uwrite!(&mut serial, "T {:x} {:x} || {} || ", buffer[0], buffer[1], check1).void_unwrap();

                xbee_data[0] = data[0];
                xbee_data[1] = data[1];
            }
            Err(_) => {
                // ufmt::uwriteln!(&mut serial, "Err\r").void_unwrap();
            }
        }

        let result = i2c.write(0x40, &[0xf5]);
        match result {
            Ok(_) => {
                // ufmt::uwriteln!(&mut serial, "Ok no hold trigger humidity reading\r").void_unwrap();
                let mut buffer = [0x00, 0x00, 0x00];
                let mut ready = false;
                while !ready {
                    delay_ms(10);
                    let read_result = i2c.read(0x40, &mut buffer);
                    ready = match read_result {
                        Ok(_) => true,
                        Err(_) => false,
                    };
                    // ufmt::uwriteln!(&mut serial, "needs delay\r").void_unwrap();
                }
                let data = buffer.as_slice().split_at(2).0;
                let check1 = htu21::check(data, &(buffer[2]));
                // ufmt::uwriteln!(&mut serial, "H {:x} {:x} || {}", buffer[0], buffer[1], check1).void_unwrap();

                xbee_data[2] = data[0];
                xbee_data[3] = data[1];
            }
            Err(_) => {
                // ufmt::uwriteln!(&mut serial, "Err\r").void_unwrap();
            }
        }


        let packet = xbee::Packet::new(xbee::ApiIdentifier::TxReq,
                                       [0x00, 0x13, 0xA2, 0x00, 0x40, 0x64, 0x03, 0x75],
                                       &xbee_data);
        for byte in packet.iter() {
            serial.write_byte(byte);
        }
        serial.flush();

        arduino_hal::delay_ms(1000);
    }
}

// let result = i2c.ping_device(0x40, Direction::Write);
// match result {
//     Ok(_) => {
//         ufmt::uwriteln!(&mut serial, "Ok\r").void_unwrap();
//     }
//     Err(_) => {
//         ufmt::uwriteln!(&mut serial, "Err\r").void_unwrap();
//     }
// }
