use core::time::Duration;

pub fn id_get() -> u64 {
    let w1 = zephyr::device_dt_get!(zephyr::dts::root().w1_gpio);

    let mut rom = zephyr::sys::w1_rom {
        family: 0,
        serial: [0; 6],
        crc: 0,
    };

    unsafe {
        zephyr::sys::w1_reset_bus(w1);
        zephyr::sys::w1_read_rom(w1, &mut rom);
        zephyr::sys::w1_rom_to_uint64(&rom)
    }
}

pub fn deep_sleep(wakeup_after: Duration) -> ! {
    unsafe {
        zephyr::sys::esp_deep_sleep(wakeup_after.as_micros() as u64);
    }
}
