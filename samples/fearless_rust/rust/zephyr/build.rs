use std::{env, fs};
use std::path::PathBuf;

fn main() {
    generate_bindings();
    import_rust_dts();
}

fn generate_bindings() {
    let zephyr_base = PathBuf::from(env::var("ZEPHYR_BASE").expect("ZEPHYR_BASE must be set!"));
    let input_path = PathBuf::from(env::var("BINDGEN_INPUT").expect("BINDGEN_INPUT must be set!"));
    let output_path = PathBuf::from(env::var("BINDGEN_OUTPUT").expect("BINDGEN_OUTPUT must be set!"));
    let clang_args_path = PathBuf::from(env::var("BINDGEN_CLANG_ARGS").expect("BINDGEN_CLANG_ARGS must be set!"));
    let clang_args = fs::read_to_string(clang_args_path).expect("Failed to read BINDGEN_CLANG_ARGS file!");
    let wrap_static_fns = PathBuf::from(env::var("BINDGEN_WRAP_STATIC_FNS").expect("BINDGEN_WRAP_STATIC_FNS must be set!"));

    // report environment variable back to allow IDE to resolve the included file
    println!("cargo:rustc-env=BINDGEN_OUTPUT={}", output_path.to_str().unwrap());

    let bindings = bindgen::Builder::default()
        .use_core()
        .layout_tests(false)
        .detect_include_paths(false)
        .wrap_static_fns(true)
        .wrap_static_fns_path(wrap_static_fns)
        .clang_args(clang_args.split(';'))
        .header(input_path.to_str().unwrap())
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))

        .allowlist_file(zephyr_base.join(".*").to_str().unwrap())
        .allowlist_file(".*/errno.h")
        .allowlist_file(".*/esp_sleep.h")
        .blocklist_function("z_impl_.*")
        .blocklist_var("K_SYSCALL_.*")
        .blocklist_var("DT_.*")
        .blocklist_var("Z_UTIL_.*")

        // Deprecated functions, hopefully there is a more generic way of doing this.
        .blocklist_function("sys_clock_timeout_end_calc")
        .blocklist_function("net_ipv6_set_hop_limit")
        .blocklist_function("net_if_ipv4_set_netmask_by_index")

        .generate()
        .expect("Unable to generate bindings!");

    bindings
        .write_to_file(output_path)
        .expect("Couldn't write bindings!");
}

fn import_rust_dts() {
    let rust_dts = PathBuf::from(env::var("DTS_RUST").expect("DTS_RUST must be set!"));

    // report environment variable back to allow IDE to resolve the included file
    println!("cargo:rustc-env=DTS_RUST={}", rust_dts.to_str().unwrap());
}
