#include <zephyr/kernel.h>
#include <zephyr/devicetree.h>
#include <stm32h743xx.h>
#include <core_cm7.h>

#define FLASH_IRQ_NUM     4
#define NUM_MPU_REGIONS   16

static void dump_mpu_regions(void) {
    uint32_t rbar[NUM_MPU_REGIONS];  // base address
    uint32_t rasr[NUM_MPU_REGIONS];  // attribute and size

    uint32_t key = irq_lock();

    uint32_t ctrl = MPU->CTRL;

    for (uint32_t i = 0; i < NUM_MPU_REGIONS; i++) {
        MPU->RNR = i;
        rbar[i] = MPU->RBAR;
        rasr[i] = MPU->RASR;
    }

    irq_unlock(key);

    printk("MPU Control: %08x\n", ctrl);

    for (uint32_t i = 0; i < NUM_MPU_REGIONS; i++) {
        if (!(rasr[i] & MPU_RASR_ENABLE_Msk)) {
            continue;
        }

        uint32_t start_address = (rbar[i] & MPU_RBAR_ADDR_Msk);
        uint32_t size_bits = (rasr[i] & MPU_RASR_SIZE_Msk) >> MPU_RASR_SIZE_Pos;
        uint32_t region_size = 1 << (size_bits + 1);
        uint32_t end_address = start_address + region_size - 1;

        uint32_t srd = (rasr[i] & MPU_RASR_SRD_Msk) >> MPU_RASR_SRD_Pos;
        uint32_t xn = (rasr[i] & MPU_RASR_XN_Msk) >> MPU_RASR_XN_Pos;
        uint32_t ap = (rasr[i] & MPU_RASR_AP_Msk) >> MPU_RASR_AP_Pos;
        uint32_t tex = (rasr[i] & MPU_RASR_TEX_Msk) >> MPU_RASR_TEX_Pos;
        uint32_t s = (rasr[i] & MPU_RASR_S_Msk) >> MPU_RASR_S_Pos;
        uint32_t c = (rasr[i] & MPU_RASR_C_Msk) >> MPU_RASR_C_Pos;
        uint32_t b = (rasr[i] & MPU_RASR_B_Msk) >> MPU_RASR_B_Pos;

        printk("MPU Region %u: 0x%08x-0x%08x: SRD=0x%02x, XN=%u, AP=0x%x, TEX=0x%x, S=%u, C=%u, B=%u\n",
               i, start_address, end_address, srd, xn, ap, tex, s, c, b);
    }
}

static void flash_error_isr(void *arg) {
    printk("ISR triggered: %u ms\n", k_uptime_get_32());
    printk("FLASH_SR1: 0x%08x\n", FLASH->SR1);
    printk("FLASH_SR2: 0x%08x\n", FLASH->SR2);

    FLASH->CCR1 = FLASH_FLAG_ALL_BANK1;
    FLASH->CCR2 = FLASH_FLAG_ALL_BANK2;
}

static void enable_flash_error_interrupt(void) {
    FLASH->CR1 |= FLASH_CR_RDSERRIE;
    FLASH->CR1 |= FLASH_CR_RDPERRIE;
    FLASH->CR2 |= FLASH_CR_RDSERRIE;
    FLASH->CR2 |= FLASH_CR_RDPERRIE;

    IRQ_CONNECT(FLASH_IRQ_NUM, 0, flash_error_isr, NULL, 0);
    irq_enable(FLASH_IRQ_NUM);
}

void main(void) {
    dump_mpu_regions();
    enable_flash_error_interrupt();

    while (1) {
        // error doesn't occur if sleep is longer than 1 tick
        k_sleep(K_TICKS(1));
    }
}
