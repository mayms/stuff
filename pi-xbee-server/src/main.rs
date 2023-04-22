use rppal::uart::{Parity, Uart};
use std::error::Error;
use std::time::Duration;
use htu21;
use log::{debug, error, info};

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let mut uart = Uart::new(57_600, Parity::None, 8, 1)?;
    uart.set_read_mode(13, Duration::default())?;
    let mut buffer = [0u8; 13];
    loop {
        if uart.read(&mut buffer)? > 0 {
            let x = buffer.as_slice().split_at(8).1.split_at(4).0;
            let temperature = htu21::parse_temperature(x.split_at(2).0);
            let humidity = htu21::parse_humidity(x.split_at(2).1);
            info!("sensor data received; temperature={} humidity={}", temperature, humidity);
        }
    }
}
