#include <esp_sleep.h>
#include <zephyr/kernel.h>
#include <zephyr/device.h>
#include <zephyr/devicetree.h>
#include <zephyr/drivers/sensor.h>
#include <zephyr/sys/poweroff.h>
#include <zephyr/logging/log.h>
#include <zephyr/logging/log_ctrl.h>
#include "wifi.h"
#include "udp.h"

LOG_MODULE_REGISTER(main);

int main(void)
{
	wifi_init();

	if (wifi_connect() != 0) {
		goto teardown;
	}

	if (udp_init() != 0) {
		goto teardown_disconnect;
	}

	const char* hello = "hello world!\n";
	(void) udp_send(hello, strlen(hello));

	// not sure how to properly wait until the UDP data has been sent
	k_sleep(K_SECONDS(3));

teardown_disconnect:
	wifi_disconnect();

teardown:
	LOG_INF("Powering down");

	while (log_process()) {
		// flush log messages
	}

	esp_sleep_enable_timer_wakeup(10 * 1000000);
	sys_poweroff();
	return 0;

	const struct device *dev = DEVICE_DT_GET(DT_ALIAS(temp_sensor));
	int res;

	while (true) {
		struct sensor_value temp;

		res = sensor_sample_fetch(dev);
		if (res != 0) {
			printk("sample_fetch() failed: %d\n", res);
			return res;
		}

		res = sensor_channel_get(dev, SENSOR_CHAN_AMBIENT_TEMP, &temp);
		if (res != 0) {
			printk("channel_get() failed: %d\n", res);
			return res;
		}

		printk("Temp: %d.%06d\n", temp.val1, temp.val2);
		k_sleep(K_MSEC(2000));
	}

	return 0;
}
