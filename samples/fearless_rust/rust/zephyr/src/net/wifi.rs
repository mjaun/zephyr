use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::ffi::c_void;
use core::mem::{MaybeUninit, size_of_val};
use core::ptr::{null, null_mut};
use core::time::Duration;
use crate::kernel::errno::{check_ptr_mut, check_result, ENODEV, ErrnoResult};
use crate::sys::{net_if_get_default, net_mgmt_NET_REQUEST_WIFI_CONNECT, net_mgmt_NET_REQUEST_WIFI_DISCONNECT};

pub struct Wifi {
    // TODO: Protect against concurrent access by Rust application and Zephyr callbacks
    data: Box<WifiData>,
}

struct WifiData {
    net_if: *mut crate::sys::net_if,
    rust_cb: crate::sys::rust_net_mgmt_cb,
    on_connected: Vec<Box<dyn FnMut(i32)>>,
    on_disconnected: Vec<Box<dyn FnMut(i32)>>,
}

#[derive(Default)]
pub struct WifiConnectReqParams {
    pub ssid: String,
    pub psk: Option<String>,
    pub sae_password: Option<String>,
    pub channel: u8,
    pub security: WifiSecurityType,
    pub band: WifiFrequencyBands,
    pub mfp: WifiMfpOptions,
    pub bssid: Option<WifiMacAddress>,
    pub timeout: Duration,
}

pub type WifiMacAddress = [u8; crate::sys::WIFI_MAC_ADDR_LEN as usize];

pub const WIFI_CHANNEL_MIN: u8 = crate::sys::WIFI_CHANNEL_MIN as u8;
pub const WIFI_CHANNEL_MAX: u8 = crate::sys::WIFI_CHANNEL_MAX as u8;
pub const WIFI_CHANNEL_ANY: u8 = crate::sys::WIFI_CHANNEL_ANY as u8;

#[repr(u32)]
#[derive(Default)]
pub enum WifiSecurityType {
    #[default]
    None = crate::sys::wifi_security_type_WIFI_SECURITY_TYPE_NONE,
    Psk = crate::sys::wifi_security_type_WIFI_SECURITY_TYPE_PSK,
    PskSha256 = crate::sys::wifi_security_type_WIFI_SECURITY_TYPE_PSK_SHA256,
    Sae = crate::sys::wifi_security_type_WIFI_SECURITY_TYPE_SAE,
    Wapi = crate::sys::wifi_security_type_WIFI_SECURITY_TYPE_WAPI,
    Eap = crate::sys::wifi_security_type_WIFI_SECURITY_TYPE_EAP,
    Wep = crate::sys::wifi_security_type_WIFI_SECURITY_TYPE_WEP,
    WpaPsk = crate::sys::wifi_security_type_WIFI_SECURITY_TYPE_WPA_PSK,
    WpaAutoPersonal = crate::sys::wifi_security_type_WIFI_SECURITY_TYPE_WPA_AUTO_PERSONAL,
}

#[repr(u32)]
#[derive(Default)]
pub enum WifiFrequencyBands {
    #[default]
    Band24Ghz = crate::sys::wifi_frequency_bands_WIFI_FREQ_BAND_2_4_GHZ,
    Band5Ghz = crate::sys::wifi_frequency_bands_WIFI_FREQ_BAND_5_GHZ,
    Band6Ghz = crate::sys::wifi_frequency_bands_WIFI_FREQ_BAND_6_GHZ,
}

#[repr(u32)]
#[derive(Default)]
pub enum WifiMfpOptions {
    Disable = crate::sys::wifi_mfp_options_WIFI_MFP_DISABLE,
    #[default]
    Optional = crate::sys::wifi_mfp_options_WIFI_MFP_OPTIONAL,
    Required = crate::sys::wifi_mfp_options_WIFI_MFP_REQUIRED,
}

const NET_WIFI_REQUEST: u32 = 0x5156_0000;
const REQUEST_WIFI_CONNECT: u32 = NET_WIFI_REQUEST | crate::sys::net_request_wifi_cmd_NET_REQUEST_WIFI_CMD_CONNECT;
const REQUEST_WIFI_DISCONNECT: u32 = NET_WIFI_REQUEST | crate::sys::net_request_wifi_cmd_NET_REQUEST_WIFI_CMD_DISCONNECT;

const NET_WIFI_EVENT: u32 = 0xD156_0000;
const EVENT_WIFI_CONNECT_RESULT: u32 = NET_WIFI_EVENT | crate::sys::net_event_wifi_cmd_NET_EVENT_WIFI_CMD_CONNECT_RESULT;
const EVENT_WIFI_DISCONNECT_RESULT: u32 = NET_WIFI_EVENT | crate::sys::net_event_wifi_cmd_NET_EVENT_WIFI_CMD_DISCONNECT_RESULT;

impl Wifi {
    pub fn from_default_iface() -> ErrnoResult<Self> {
        unsafe {
            let ret = net_if_get_default();
            Ok(Wifi::new(check_ptr_mut(ret, ENODEV)?))
        }
    }

    fn new(net_if: *mut crate::sys::net_if) -> Self {
        let data = Box::new(WifiData {
            net_if,
            rust_cb: Default::default(),
            on_connected: Default::default(),
            on_disconnected: Default::default(),
        });

        let data_ptr = Box::into_raw(data);

        unsafe {
            crate::sys::rust_net_mgmt_add_event_callback(
                &mut (*data_ptr).rust_cb as *mut crate::sys::rust_net_mgmt_cb,
                EVENT_WIFI_CONNECT_RESULT | EVENT_WIFI_DISCONNECT_RESULT,
                data_ptr as *mut c_void,
            );

            Wifi { data: Box::from_raw(data_ptr) }
        }
    }

    pub fn connect(&mut self, params: WifiConnectReqParams) -> ErrnoResult<()> {
        let mut mapped_params = crate::sys::wifi_connect_req_params {
            ssid: params.ssid.as_ptr(),
            ssid_length: params.ssid.len() as u8,
            psk: match &params.psk {
                Some(psk) => psk.as_ptr(),
                None => null(),
            },
            psk_length: match &params.psk {
                Some(psk) => psk.len() as u8,
                None => 0,
            },
            sae_password: match &params.sae_password {
                Some(sae_password) => sae_password.as_ptr(),
                None => null(),
            },
            sae_password_length: match &params.sae_password {
                Some(sae_password) => sae_password.len() as u8,
                None => 0,
            },
            band: params.band as u8,
            channel: params.channel,
            security: params.security as u32,
            mfp: params.mfp as u32,
            bssid: match &params.bssid {
                Some(bssid) => *bssid,
                None => WifiMacAddress::default(),
            },
            timeout: match params.timeout {
                Duration::MAX => crate::sys::SYS_FOREVER_MS,
                value => value.as_secs() as i32,
            },
        };

        let req_size = size_of_val(&mapped_params);
        let req_data: *mut crate::sys::wifi_connect_req_params = &mut mapped_params;

        unsafe {
            let ret = net_mgmt_NET_REQUEST_WIFI_CONNECT(
                REQUEST_WIFI_CONNECT,
                self.data.net_if,
                req_data as *mut c_void,
                req_size,
            );

            check_result(ret)
        }
    }

    pub fn disconnect(&mut self) -> ErrnoResult<()> {
        unsafe {
            let ret = net_mgmt_NET_REQUEST_WIFI_DISCONNECT(
                REQUEST_WIFI_DISCONNECT,
                self.data.net_if,
                null_mut(),
                0,
            );

            check_result(ret)
        }
    }

    pub fn on_connected<F>(&mut self, callback: F)
    where
        F: FnMut(i32) + Send + 'static,
    {
        self.data.on_connected.push(Box::new(callback));
    }

    pub fn on_disconnected<F>(&mut self, callback: F)
    where
        F: FnMut(i32) + Send + 'static,
    {
        self.data.on_disconnected.push(Box::new(callback));
    }
}

impl Drop for Wifi {
    fn drop(&mut self) {
        unsafe {
            crate::sys::rust_net_mgmt_del_event_callback(
                &mut self.data.rust_cb
            );
        }
    }
}

#[no_mangle]
extern "C" fn rust_net_mgmt_event_handler(rust_data: *mut c_void,
                                          iface: *mut crate::sys::net_if,
                                          mgmt_event: u32,
                                          info: *const c_void,
                                          _info_length: usize) {
    let mut data = unsafe {
        Box::from_raw(rust_data as *mut WifiData)
    };

    if iface != data.net_if {
        return;
    }

    let get_status = |info: *const c_void| -> i32 {
        let status = info as *const crate::sys::wifi_status;
        unsafe { (*status).__bindgen_anon_1.status }
    };

    match mgmt_event {
        EVENT_WIFI_CONNECT_RESULT => {
            for callback in data.on_connected.iter_mut() {
                callback(get_status(info));
            }
        }
        EVENT_WIFI_DISCONNECT_RESULT => {
            for callback in data.on_disconnected.iter_mut() {
                callback(get_status(info));
            }
        }
        _ => ()
    }

    // turn back into a raw pointer to avoid deleting
    Box::into_raw(data);
}

impl Default for crate::sys::rust_net_mgmt_cb {
    fn default() -> Self {
        // data is initialized when adding the event callback
        let uninit: MaybeUninit<Self> = MaybeUninit::uninit();
        unsafe { uninit.assume_init() }
    }
}
