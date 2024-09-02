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

	LOG_INF("Device ID: %016llx", measurement.device_id);
	LOG_INF("Temperature: %.3f C", (double) measurement.value);

	LOG_INF("Starting to upload data");

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

	sys_poweroff();
}
