#![no_std]

extern crate zephyr;
extern crate alloc;

mod device;

use alloc::string::String;
use core::time::Duration;
use zephyr::drivers::sensor::{Sensor, SensorChannel};
use zephyr::{device_dt_get, dt_alias, printkln};
use zephyr::net::wifi::{Wifi, WifiConnectReqParams, WifiSecurityType};
use zephyr::kernel::sleep;
use zephyr::net::socket::UdpSocket;

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
    let data = "Hello World!".as_bytes();

    udp.bind("0.0.0.0:0").unwrap();
    udp.sendto(data, env!("SERVER_ADDRESS")).unwrap();

    sleep(Duration::from_secs(5));

    printkln!("Disconnecting...");

    wifi.disconnect().unwrap();

    sleep(Duration::from_secs(5));

    printkln!("Done!");

    device::deep_sleep(Duration::from_secs(10));
}
