// Copyright (c) 2024 Zühlke Engineering AG
// SPDX-License-Identifier: Apache-2.0

use std::{env, fs};
use std::path::PathBuf;

fn main() {
    let input_header = env::current_dir().unwrap().join("bindgen_input.h");
    let out_path = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR must be set!"));
    let zephyr_base = PathBuf::from(env::var("ZEPHYR_BASE").expect("ZEPHYR_BASE must be set!"));
    let wrap_static_fns = PathBuf::from(env::var("BINDGEN_WRAP_STATIC_FNS").expect("BINDGEN_WRAP_STATIC_FNS must be set!"));
    let clang_args_path = PathBuf::from(env::var("BINDGEN_CLANG_ARGS").expect("BINDGEN_CLANG_ARGS must be set!"));
    let clang_args = fs::read_to_string(clang_args_path).expect("Failed to read BINDGEN_CLANG_ARGS file!");

    let bindings = bindgen::Builder::default()
        .use_core()
        .layout_tests(false)
        .detect_include_paths(false)
        .wrap_static_fns(true)
        .wrap_static_fns_path(wrap_static_fns)
        .clang_args(clang_args.split(';'))
        .header(input_header.to_str().unwrap())
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))

        .allowlist_file(zephyr_base.join(".*").to_str().unwrap())
        .allowlist_file(".*/errno.h")
        .blocklist_function("z_impl_.*")
        .blocklist_var("K_SYSCALL_.*")
        .blocklist_var("DT_.*")
        .blocklist_var("CONFIG_.*")
        .blocklist_var("Z_UTIL_.*")

        // Deprecated function, hopefully there is a more generic way of doing this.
        .blocklist_function("sys_clock_timeout_end_calc")

        .generate()
        .expect("Unable to generate bindings!");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
