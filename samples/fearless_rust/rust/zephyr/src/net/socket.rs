use alloc::ffi::CString;
use core::ffi::{c_int, c_void};
use core::mem::{MaybeUninit, size_of_val};
use crate::kernel::errno::{check_result_errno, check_value_errno, EINVAL, ENOMEM, ErrnoResult};
use crate::sys::zsock_close;

pub struct UdpSocket {
    socket: c_int,
}

impl UdpSocket {
    pub fn new() -> ErrnoResult<Self> {
        unsafe {
            let socket = crate::sys::zsock_socket(
                crate::sys::AF_INET as c_int,
                crate::sys::net_sock_type_SOCK_DGRAM as c_int,
                crate::sys::net_ip_protocol_IPPROTO_UDP as c_int,
            );

            Ok(Self { socket: check_value_errno(socket)? as c_int })
        }
    }

    pub fn bind(&mut self, addr: &str) -> ErrnoResult<()> {
        let sockaddr = parse_address(addr)?;

        unsafe {
            let ret = crate::sys::zsock_bind(
                self.socket,
                (&sockaddr as *const crate::sys::sockaddr_in) as *const crate::sys::sockaddr,
                size_of_val(&sockaddr)
            );

            check_result_errno(ret)
        }
    }

    pub fn sendto(&mut self, buf: &[u8], target: &str) -> ErrnoResult<u32> {
        let sockaddr = parse_address(target)?;

        unsafe {
            let ret = crate::sys::zsock_sendto(
                self.socket,
                buf.as_ptr() as *const c_void,
                buf.len(),
                0,
                (&sockaddr as *const crate::sys::sockaddr_in) as *const crate::sys::sockaddr,
                size_of_val(&sockaddr)
            );

            check_value_errno(ret as i32)
        }
    }
}

impl Drop for UdpSocket {
    fn drop(&mut self) {
        unsafe {
            zsock_close(self.socket);
        }
    }
}

fn parse_address(addr: &str) -> ErrnoResult<crate::sys::sockaddr_in> {
    let mut parts = addr.split(':');
    let ip_str = parts.next().ok_or(EINVAL)?;
    let port_str = parts.next().ok_or(EINVAL)?;

    let port = port_str.parse::<u16>().map_err(|_| { EINVAL })?;

    let mut ret = crate::sys::sockaddr_in {
        sin_family: crate::sys::AF_INET as crate::sys::sa_family_t,
        sin_port: port.to_be(),
        sin_addr: unsafe { MaybeUninit::uninit().assume_init() },
    };

    let ip_cstr = CString::new(ip_str).map_err(|_| { ENOMEM })?;

    unsafe {
        crate::sys::zsock_inet_pton(
            ret.sin_family,
            ip_cstr.as_ptr(),
            (&mut ret.sin_addr as *mut crate::sys::in_addr) as *mut c_void,
        );
    }

    Ok(ret)
}
