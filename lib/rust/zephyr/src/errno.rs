// Copyright (c) 2024 Zühlke Engineering AG
// SPDX-License-Identifier: Apache-2.0

use int_enum::IntEnum;

#[repr(u32)]
#[derive(Debug, PartialEq, IntEnum)]
pub enum Errno {
    EPERM = zephyr_sys::EPERM,
    ENOENT = zephyr_sys::ENOENT,
    ESRCH = zephyr_sys::ESRCH,
    EINTR = zephyr_sys::EINTR,
    EIO = zephyr_sys::EIO,
    ENXIO = zephyr_sys::ENXIO,
    E2BIG = zephyr_sys::E2BIG,
    ENOEXEC = zephyr_sys::ENOEXEC,
    EBADF = zephyr_sys::EBADF,
    ECHILD = zephyr_sys::ECHILD,
    EAGAIN = zephyr_sys::EAGAIN,
    ENOMEM = zephyr_sys::ENOMEM,
    EACCES = zephyr_sys::EACCES,
    EFAULT = zephyr_sys::EFAULT,
    ENOTBLK = zephyr_sys::ENOTBLK,
    EBUSY = zephyr_sys::EBUSY,
    EEXIST = zephyr_sys::EEXIST,
    EXDEV = zephyr_sys::EXDEV,
    ENODEV = zephyr_sys::ENODEV,
    ENOTDIR = zephyr_sys::ENOTDIR,
    EISDIR = zephyr_sys::EISDIR,
    EINVAL = zephyr_sys::EINVAL,
    ENFILE = zephyr_sys::ENFILE,
    EMFILE = zephyr_sys::EMFILE,
    ENOTTY = zephyr_sys::ENOTTY,
    ETXTBSY = zephyr_sys::ETXTBSY,
    EFBIG = zephyr_sys::EFBIG,
    ENOSPC = zephyr_sys::ENOSPC,
    ESPIPE = zephyr_sys::ESPIPE,
    EROFS = zephyr_sys::EROFS,
    EMLINK = zephyr_sys::EMLINK,
    EPIPE = zephyr_sys::EPIPE,
    EDOM = zephyr_sys::EDOM,
    ERANGE = zephyr_sys::ERANGE,
    ENOMSG = zephyr_sys::ENOMSG,
    EDEADLK = zephyr_sys::EDEADLK,
    ENOLCK = zephyr_sys::ENOLCK,
    ENOSTR = zephyr_sys::ENOSTR,
    ENODATA = zephyr_sys::ENODATA,
    ETIME = zephyr_sys::ETIME,
    ENOSR = zephyr_sys::ENOSR,
    EPROTO = zephyr_sys::EPROTO,
    EBADMSG = zephyr_sys::EBADMSG,
    ENOSYS = zephyr_sys::ENOSYS,
    ENOTEMPTY = zephyr_sys::ENOTEMPTY,
    ENAMETOOLONG = zephyr_sys::ENAMETOOLONG,
    ELOOP = zephyr_sys::ELOOP,
    EOPNOTSUPP = zephyr_sys::EOPNOTSUPP,
    EPFNOSUPPORT = zephyr_sys::EPFNOSUPPORT,
    ECONNRESET = zephyr_sys::ECONNRESET,
    ENOBUFS = zephyr_sys::ENOBUFS,
    EAFNOSUPPORT = zephyr_sys::EAFNOSUPPORT,
    EPROTOTYPE = zephyr_sys::EPROTOTYPE,
    ENOTSOCK = zephyr_sys::ENOTSOCK,
    ENOPROTOOPT = zephyr_sys::ENOPROTOOPT,
    ESHUTDOWN = zephyr_sys::ESHUTDOWN,
    ECONNREFUSED = zephyr_sys::ECONNREFUSED,
    EADDRINUSE = zephyr_sys::EADDRINUSE,
    ECONNABORTED = zephyr_sys::ECONNABORTED,
    ENETUNREACH = zephyr_sys::ENETUNREACH,
    ENETDOWN = zephyr_sys::ENETDOWN,
    ETIMEDOUT = zephyr_sys::ETIMEDOUT,
    EHOSTDOWN = zephyr_sys::EHOSTDOWN,
    EHOSTUNREACH = zephyr_sys::EHOSTUNREACH,
    EINPROGRESS = zephyr_sys::EINPROGRESS,
    EALREADY = zephyr_sys::EALREADY,
    EDESTADDRREQ = zephyr_sys::EDESTADDRREQ,
    EMSGSIZE = zephyr_sys::EMSGSIZE,
    EPROTONOSUPPORT = zephyr_sys::EPROTONOSUPPORT,
    ESOCKTNOSUPPORT = zephyr_sys::ESOCKTNOSUPPORT,
    EADDRNOTAVAIL = zephyr_sys::EADDRNOTAVAIL,
    ENETRESET = zephyr_sys::ENETRESET,
    EISCONN = zephyr_sys::EISCONN,
    ENOTCONN = zephyr_sys::ENOTCONN,
    ETOOMANYREFS = zephyr_sys::ETOOMANYREFS,
    ENOTSUP = zephyr_sys::ENOTSUP,
    EILSEQ = zephyr_sys::EILSEQ,
    EOVERFLOW = zephyr_sys::EOVERFLOW,
    ECANCELED = zephyr_sys::ECANCELED,
}

pub type ZephyrResult<T> = Result<T, Errno>;

pub fn check_result(result: core::ffi::c_int) -> ZephyrResult<()> {
    check_value(result).map(|_| ())
}

pub fn check_value(result: core::ffi::c_int) -> ZephyrResult<i32> {
    if result >= 0 {
        return Ok(result as i32);
    }

    match Errno::try_from(-result as u32) {
        Ok(errno) => Err(errno),
        _ => panic!("Unexpected value"),
    }
}

pub fn check_ptr<T>(result: *mut T, null_error: Errno) -> ZephyrResult<*mut T> {
    if result == core::ptr::null_mut() {
        Err(null_error)
    } else {
        Ok(result)
    }
}
