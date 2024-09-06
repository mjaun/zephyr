use alloc::string::String;
use core::ffi::c_void;
use core::mem::{size_of_val};
use core::ptr::{null, null_mut};
use core::time::Duration;
use crate::kernel::errno::{check_ptr_mut, check_result, ENODEV, ErrnoResult};
use crate::sys::{net_if_get_default, net_mgmt_NET_REQUEST_WIFI_CONNECT, net_mgmt_NET_REQUEST_WIFI_DISCONNECT};

pub struct Wifi {
    net_if: *mut crate::sys::net_if,
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

const NET_WIFI_BASE: u32 = 0x51560000;
const REQUEST_WIFI_CONNECT: u32 = NET_WIFI_BASE | crate::sys::net_request_wifi_cmd_NET_REQUEST_WIFI_CMD_CONNECT;
const REQUEST_WIFI_DISCONNECT: u32 = NET_WIFI_BASE | crate::sys::net_request_wifi_cmd_NET_REQUEST_WIFI_CMD_DISCONNECT;

impl Wifi {
    pub fn get_default() -> ErrnoResult<Self> {
        unsafe {
            let ret = net_if_get_default();
            Ok(Wifi { net_if: check_ptr_mut(ret, ENODEV)? })
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
                self.net_if,
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
                self.net_if,
                null_mut(),
                0
            );

            check_result(ret)
        }
    }
}
