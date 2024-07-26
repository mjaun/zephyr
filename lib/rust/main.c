/*
 * Copyright (c) 2024 Linaro LTD
 * SPDX-License-Identifier: Apache-2.0
 */

/* This main is brought into the Rust application. */
#include <zephyr/kernel.h>
#include <zephyr/app_memory/app_memdomain.h>

#ifdef CONFIG_RUST

extern void rust_main(void);

#ifdef CONFIG_USERSPACE

K_APPMEM_PARTITION_DEFINE(rust_mem_part);

K_APP_BMEM(rust_mem_part) uint8_t rust_heap_buf[CONFIG_RUST_HEAP_SIZE];
K_APP_BMEM(rust_mem_part) struct k_heap rust_heap;

struct k_mem_domain rust_mem_domain;

#else

K_HEAP_DEFINE(rust_heap, CONFIG_RUST_HEAP_SIZE);

#endif

int main(void)
{
	k_tid_t main_thread = k_current_get();

#ifdef CONFIG_USERSPACE
	k_mem_domain_init(&rust_mem_domain, 0, NULL);

	for (uint8_t i = 0; i < k_mem_domain_default.num_partitions; i++) {
		k_mem_domain_add_partition(&rust_mem_domain, &k_mem_domain_default.partitions[i]);
	}

	k_mem_domain_add_partition(&rust_mem_domain, &rust_mem_part);
	k_mem_domain_add_thread(&rust_mem_domain, main_thread);

	k_heap_init(&rust_heap, rust_heap_buf, sizeof(rust_heap_buf));
#endif

	k_thread_heap_assign(main_thread, &rust_heap);

	rust_main();
	return 0;
}

#endif
