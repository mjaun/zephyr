#![no_std]

extern crate zephyr;
extern crate alloc;

mod device;

use alloc::format;
use alloc::string::String;
use core::time::Duration;
use zephyr::drivers::sensor::{Sensor, SensorChannel};
use zephyr::{device_dt_get, dt_alias, printkln};
use zephyr::net::wifi::{Wifi, WifiConnectReqParams, WifiSecurityType};
use zephyr::kernel::sleep;
use zephyr::net::socket::UdpSocket;
use serde::Serialize;
use zephyr::kernel::errno::ErrnoResult;

const DEEP_SLEEP_INTERVAL: Duration = Duration::from_secs(10);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
const UDP_SEND_TIMEOUT: Duration = Duration::from_secs(3);
const DISCONNECT_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Serialize)]
struct Measurement {
    device_id: String,
    temperature: f32,
}

#[no_mangle]
extern "C" fn rust_main() {
    let measurement = perform_measurement();
    let data = serialize_measurement(&measurement);
    upload_data(data.as_str()).ok();

    printkln!("Entering deep sleep...");
    device::deep_sleep(DEEP_SLEEP_INTERVAL);
}

fn perform_measurement() -> Measurement {
    let device_id = format!("{:X}", device::id_get());
    printkln!("device_id={}", device_id);

    let mut sensor = Sensor::new(device_dt_get!(dt_alias!(temp_sensor)));
    sensor.sample_fetch().unwrap();
    let value = sensor.channel_get(SensorChannel::AmbientTemp).unwrap();
    let temperature: f32 = value.into();
    printkln!("temperature={}", temperature);

    Measurement { device_id, temperature }
}

fn serialize_measurement(measurement: &Measurement) -> String {
    let json = serde_json::to_string(&measurement).unwrap();
    printkln!("json={}", json);
    json
}

fn upload_data(data: &str) -> ErrnoResult<()> {
    let wifi = Wifi::from_default_iface()?;

    printkln!("Connecting...");
    wifi.connect(WifiConnectReqParams {
        ssid: String::from(env!("WIFI_SSID")),
        psk: Some(String::from(env!("WIFI_PSK"))),
        security: WifiSecurityType::Psk,
        timeout: CONNECT_TIMEOUT,
        ..Default::default()
    })?;

    printkln!("Sending data...");
    let mut udp = UdpSocket::new().unwrap();
    udp.bind("0.0.0.0:0")?;
    udp.sendto(data.as_bytes(), env!("SERVER_ADDRESS"))?;

    // not sure how to properly wait until the UDP data has been sent
    sleep(UDP_SEND_TIMEOUT);

    printkln!("Disconnecting...");
    wifi.disconnect(DISCONNECT_TIMEOUT)?;

    Ok(())
}
