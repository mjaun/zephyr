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

#[derive(Serialize)]
struct Measurement {
    device_id: String,
    temperature: f32,
}

#[no_mangle]
extern "C" fn rust_main() {
    printkln!("Hello World!");

    let device_id = device::id_get();
    printkln!("device_id={}", device_id);

    let mut sensor = Sensor::new(device_dt_get!(dt_alias!(temp_sensor)));
    sensor.sample_fetch().unwrap();
    let value = sensor.channel_get(SensorChannel::AmbientTemp).unwrap();
    let temperature: f32 = value.into();

    printkln!("temperature={}", temperature);

    let measurement = Measurement {
        device_id: format!("{:X}", device_id),
        temperature,
    };

    let data = serde_json::to_string(&measurement).unwrap();
    printkln!("json={}", data);

    let mut wifi = Wifi::from_default_iface().unwrap();

    wifi.on_connected(|status| { printkln!("Connected: {}", status); });
    wifi.on_disconnected(|status| { printkln!("Disconnected: {}", status); });

    printkln!("Connecting...");

    wifi.connect(WifiConnectReqParams {
        ssid: String::from(env!("WIFI_SSID")),
        psk: Some(String::from(env!("WIFI_PSK"))),
        security: WifiSecurityType::Psk,
        ..Default::default()
    }).unwrap();

    sleep(Duration::from_secs(10));

    printkln!("Sending data...");

    let mut udp = UdpSocket::new().unwrap();

    udp.bind("0.0.0.0:0").unwrap();
    udp.sendto(data.as_bytes(), env!("SERVER_ADDRESS")).unwrap();

    sleep(Duration::from_secs(5));

    printkln!("Disconnecting...");

    wifi.disconnect().unwrap();

    sleep(Duration::from_secs(5));

    printkln!("Done!");

    device::deep_sleep(Duration::from_secs(10));
}
