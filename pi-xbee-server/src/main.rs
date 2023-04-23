use rppal::uart::{Parity, Uart};
use std::time::Duration;
use htu21;
use log::{info};
use lazy_static::lazy_static;
use prometheus::*;
use warp::{Filter, http};
use warp::http::header::CONTENT_TYPE;

lazy_static! {
    static ref TEMPERATURE_GAUGE: Gauge = register_gauge!(opts!(
        "pi_xbee_temperature_celcius",
        "Temperature in celcius.",
        labels! {"handler" => "all",}
    ))
    .unwrap();
    static ref HUMIDITY_GAUGE: Gauge = register_gauge!(opts!(
        "pi_xbee_humidity",
        "Humidity.",
        labels! {"handler" => "all",}
    ))
    .unwrap();
}

#[tokio::main]
async fn main() {
    env_logger::init();

    tokio::spawn(async {
        let mut uart = Uart::new(57_600, Parity::None, 8, 1)
            .expect("unable to create UART");
        uart.set_read_mode(13, Duration::default())
            .expect("unable to set baud rate");
        let mut buffer = [0u8; 13];
        loop {
            if uart.read(&mut buffer).expect("unable to read") > 0 {
                let x = buffer.as_slice().split_at(8).1.split_at(4).0;
                let temperature = htu21::parse_temperature(x.split_at(2).0);
                let humidity = htu21::parse_humidity(x.split_at(2).1);
                TEMPERATURE_GAUGE.set(temperature as f64);
                HUMIDITY_GAUGE.set(humidity as f64);
                info!("sensor data received; temperature={} humidity={}", temperature, humidity);
            }
        }
    });

    let metrics = warp::path!("metrics")
        .map(|| {
            let encoder = TextEncoder::new();
            let metric_families = prometheus::gather();
            let mut buffer = vec![];
            encoder.encode(&metric_families, &mut buffer).unwrap();
            let response = http::Response::builder()
                .status(200)
                .header(CONTENT_TYPE, encoder.format_type())
                .body(buffer)
                .unwrap();
            return response;
        });

    let routes = warp::get().and(metrics);

    warp::serve(routes)
        .run(([0, 0, 0, 0], 8080))
        .await;
}
