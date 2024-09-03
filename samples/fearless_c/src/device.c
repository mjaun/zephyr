#include "device.h"
#include <zephyr/drivers/w1.h>
#include <zephyr/logging/log.h>
#include <zephyr/sys/poweroff.h>
#include <esp_sleep.h>

LOG_MODULE_REGISTER(device);

uint64_t device_id_get()
{
    const struct device* w1 = DEVICE_DT_GET(DT_NODELABEL(w1));
    struct w1_rom rom = {0};

    (void) w1_reset_bus(w1);
    (void) w1_read_rom(w1, &rom);

    return w1_rom_to_uint64(&rom);
}

void device_schedule_wakeup(int seconds)
{
    esp_sleep_enable_timer_wakeup(seconds * 1000000ULL);
}

void device_deep_sleep()
{
    LOG_INF("Entering deep sleep");
	sys_poweroff();
}
