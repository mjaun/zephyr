#include <zephyr/kernel.h>
#include <zephyr/logging/log.h>
#include <zephyr/net/wifi_mgmt.h>

extern void rust_main();

int main(void)
{
	static const uint32_t value = NET_EVENT_WIFI_CONNECT_RESULT;

	rust_main();
	return 0;
}
