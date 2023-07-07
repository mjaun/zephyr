#include <zephyr/kernel.h>
#include <zephyr/storage/flash_map.h>

void main(void) {
    printk("Hello World!\n");

    const struct flash_area *fa;
    uint8_t data[4];
    int ret;

    while (1) {
        ret = flash_area_open(FIXED_PARTITION_ID(slot0_partition), &fa);

        if (ret != 0) {
            printk("Error open: %d\n", ret);
            continue;
        }

        ret = flash_area_read(fa, 0, data, sizeof(data));

        if (ret < 0) {
            printk("Error read: %d\n", ret);
        }

        flash_area_close(fa);
        k_sleep(K_NSEC(1));
    }
}
