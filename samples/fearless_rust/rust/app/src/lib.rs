#![no_std]

extern crate zephyr;
extern crate alloc;

use zephyr::drivers::sensor::{Sensor, SensorChannel};
use zephyr::{device_dt_get, dt_alias, printkln};

#[no_mangle]
extern "C" fn rust_main() {
    printkln!("Hello World!");

    let mut sensor = Sensor::new(device_dt_get!(dt_alias!(temp_sensor)));
    sensor.sample_fetch().unwrap();
    let value = sensor.channel_get(SensorChannel::AmbientTemp).unwrap();
    let temperature: f32 = value.into();

    printkln!("temperature={}", temperature);
}
