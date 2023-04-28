#![no_std]

use crc::{Algorithm, Crc};
use crate::Error::{NotHumiditySensorValue, NotTemperatureSensoreValue};

const CUSTOM_ALG: Algorithm<u16> =
    Algorithm {
        width: 8,
        poly: 0b000000100110001,
        init: 0x0000,
        refin: false,
        refout: false,
        xorout: 0x0000,
        check: 0x0000,
        residue: 0x0000,
    };

#[derive(PartialEq, Debug)]
pub enum Error {
    NotTemperatureSensoreValue,
    NotHumiditySensorValue,
}

fn crc() -> Crc<u16> {
    return Crc::<u16>::new(&CUSTOM_ALG);
}

pub fn check(data: &[u8], check: &u8) -> bool {
    let checksum = crc().checksum(data);
    return checksum as u8 == *check;
}

pub fn parse_temperature(data: &[u8]) -> Result<f32, Error> {
    let status: u8 = (data[1] & 0b0000_0010) >> 1;
    if status != 0 {
        return Err(NotTemperatureSensoreValue);
    }
    let msb: u8 = data[0];
    let lsb: u8 = data[1] & 0b1111_1100;
    let data_u16 = u16::from_be_bytes([msb, lsb]);
    let data_f32 = f32::from(data_u16);
    return Ok(-46.85 + 175.72 * (data_f32 / 65536.0));
}

pub fn parse_humidity(data: &[u8]) -> Result<f32, Error> {
    let status: u8 = (data[1] & 0b0000_0010) >> 1;
    if status != 1 {
        return Err(NotHumiditySensorValue);
    }
    let msb: u8 = data[0];
    let lsb: u8 = data[1] & 0b1111_1100;
    let data_u16 = u16::from_be_bytes([msb, lsb]);
    let data_f32 = f32::from(data_u16);
    return Ok(-6.0 + 125.0 * (data_f32 / 65536.0));
}

#[cfg(test)]
mod tests {
    use crate::{check, crc, parse_humidity, parse_temperature};

    #[test]
    fn humidity_example() {
        let actual = crc().checksum(&[0x68, 0x3a]);
        assert_eq!(actual, 0x7c);
    }

    #[test]
    fn humidity_example_check() {
        let actual = check(&[0x68, 0x3a], &0x7c);
        assert_eq!(actual, true);
    }

    #[test]
    fn humidity_example_interpretation() {
        let actual = parse_humidity(&[0x68, 0x3a]);
        assert_eq!(actual, Ok(44.88806));
    }

    #[test]
    fn temperature_example() {
        let actual = crc().checksum(&[0x4e, 0x85]);
        assert_eq!(actual, 0x6b);
    }

    #[test]
    fn temperature_example_check() {
        let actual = check(&[0x4e, 0x85], &0x6b);
        assert_eq!(actual, true);
    }

    #[test]
    fn temperature_example_interpretation() {
        let actual = parse_temperature(&[0x4e, 0x85]);
        assert_eq!(actual, Ok(7.0436172));
    }
}
