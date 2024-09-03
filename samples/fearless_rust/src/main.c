#include "wifi.h"
#include "udp.h"
#include "measurement.h"
#include "serializer.h"
#include "device.h"
#include <zephyr/kernel.h>
#include <zephyr/logging/log.h>

LOG_MODULE_REGISTER(main);

int main(void)
{
	device_schedule_wakeup(10);

	uint64_t device_id = device_id_get();
	float temperature;

	if (measurement_get(&temperature) != 0) {
		goto teardown;
	}

	LOG_INF("Device ID:   0x%llx", device_id);
	LOG_INF("Temperature: %.3f C", (double) temperature);

	struct measurement measurement = {
		.device_id = device_id,
		.temperature = temperature,
	};

	char measurement_json[256];

	if (serializer_encode_measurement(&measurement, measurement_json, sizeof(measurement_json)) != 0) {
		goto teardown;
	}

	LOG_INF("%s", measurement_json);

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
	device_deep_sleep();
}
