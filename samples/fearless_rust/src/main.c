#include <zephyr/kernel.h>
#include <zephyr/logging/log.h>
#include <zephyr/net/wifi_mgmt.h>

LOG_MODULE_REGISTER(main);

extern void rust_main();

void rust_panic(void)
{
	k_panic();
}

int main(void)
{
	static const uint32_t value = NET_REQUEST_WIFI_DISCONNECT;

	rust_main();
	return 0;
}
