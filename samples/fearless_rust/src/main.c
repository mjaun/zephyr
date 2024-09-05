#include <zephyr/kernel.h>
#include <zephyr/logging/log.h>

LOG_MODULE_REGISTER(main);

extern void rust_main();

void rust_panic(void)
{
	k_panic();
}

int main(void)
{
	rust_main();
	return 0;
}
