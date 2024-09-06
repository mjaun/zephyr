use crate::kernel::errno::{check_result, ErrnoResult};
use crate::sys::{device, sensor_channel, sensor_channel_get, sensor_sample_fetch, sensor_value};

pub struct Sensor {
    device: *const device,
}

pub type SensorValue = sensor_value;

#[repr(u32)]
pub enum SensorChannel {
    AmbientTemp = crate::sys::sensor_channel_SENSOR_CHAN_AMBIENT_TEMP,
}

impl Sensor {
    pub fn new(device: *const device) -> Self {
        Sensor { device }
    }

    pub fn sample_fetch(&mut self) -> ErrnoResult<()> {
        unsafe {
            let ret = sensor_sample_fetch(self.device);
            check_result(ret)
        }
    }

    pub fn channel_get(&mut self, channel: SensorChannel) -> ErrnoResult<SensorValue> {
        unsafe {
            let mut val = SensorValue { val1: 0, val2: 0 };
            let ret = sensor_channel_get(self.device, channel as sensor_channel, &mut val);
            check_result(ret).map(|_| { val })
        }
    }
}

impl Into<f32> for SensorValue {
    fn into(self) -> f32 {
        self.val1 as f32 + (self.val2 as f32 * 0.000001f32)
    }
}
