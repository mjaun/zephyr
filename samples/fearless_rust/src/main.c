#include <zephyr/kernel.h>
#include <zephyr/sys/poweroff.h>
#include <zephyr/logging/log.h>
#include <zephyr/logging/log_ctrl.h>
#include <esp_sleep.h>
#include "wifi.h"
#include "udp.h"
#include "measurement.h"

LOG_MODULE_REGISTER(main);

int main(void)
{
	esp_sleep_enable_timer_wakeup(10 * 1000000);

	LOG_INF("Starting measurement");

	struct measurement measurement;

	if (measurement_get(&measurement) != 0) {
		goto teardown;
	}

	LOG_INF("Device ID:   %s", measurement.device_id);
	LOG_INF("Temperature: %.3f C", (double) measurement.temperature);

	char measurement_json[256];

	if (measurement_serialize(&measurement, measurement_json, sizeof(measurement_json)) != 0) {
		goto teardown;
	}

	LOG_INF("%s", measurement_json);

	LOG_INF("Starting to upload data");

	wifi_init();

	if (wifi_connect() != 0) {
		goto teardown;
	}

	if (udp_init() != 0) {
		goto disconnect;
	}

	(void) udp_send(measurement_json, strlen(measurement_json));

	// not sure how to properly wait until the UDP data has been sent
	k_sleep(K_SECONDS(3));

disconnect:
	wifi_disconnect();

teardown:
	LOG_INF("Powering down");
	sys_poweroff();
}
