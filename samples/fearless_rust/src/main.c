#include <zephyr/kernel.h>
#include <zephyr/logging/log.h>

LOG_MODULE_REGISTER(main);

extern void rust_main();

int main(void)
{
	rust_main();
	return 0;
}
