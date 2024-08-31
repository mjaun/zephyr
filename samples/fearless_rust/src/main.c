#include <zephyr/kernel.h>
#include <zephyr/device.h>
#include <zephyr/devicetree.h>
#include <zephyr/drivers/sensor.h>
#include "wifi.h"

int main(void)
{
	wifi_init();
	wifi_connect();

	k_sleep(K_SECONDS(10));

	wifi_disconnect();
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
