#![allow(dead_code)]

use core::include;
use core::env;

include!(env!("DTS_RUST"));

pub const fn root() -> &'static DtNode0 { &DT_NODE_0 }

#[macro_export]
macro_rules! dt_alias {
    ($alias:ident) => { $crate::dts::root().aliases.$alias };
}

#[macro_export]
macro_rules! device_dt_get {
    ($node:expr) => { $node.device() };
}
