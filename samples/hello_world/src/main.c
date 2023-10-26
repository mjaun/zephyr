/*
 * Copyright (c) 2020 Intel Corporation
 *
 * SPDX-License-Identifier: Apache-2.0
 */

#include <stdlib.h>
#include <string.h>
#include <zephyr/kernel.h>

#include "bh_platform.h"
#include "wasm_export.h"

#include <zephyr/logging/log.h>
LOG_MODULE_REGISTER(main);

#define APP_STACK_SIZE 8192
#define APP_HEAP_SIZE 8192

static char wasm_heap_buf[KB(256)];
static struct k_heap wasm_heap;

static uint8_t wasm_binary[] = {
#include "test.wasm.inc"
};

static void * wasm_malloc(unsigned int size) {
	// align to 8 bytes
	if (size & 0x7) {
		size &= ~0x7;
		size += 0x8;
	}

	return k_heap_aligned_alloc(&wasm_heap, 8, size, K_NO_WAIT);
}

static void wasm_free(void *ptr) {
	return k_heap_free(&wasm_heap, ptr);
}

int main() {
	char error_buf[128];

	k_heap_init(&wasm_heap, wasm_heap_buf, sizeof(wasm_heap_buf));

	RuntimeInitArgs init_args;
	memset(&init_args, 0, sizeof(RuntimeInitArgs));
	init_args.mem_alloc_type = Alloc_With_Allocator;
	init_args.mem_alloc_option.allocator.malloc_func = wasm_malloc;
	init_args.mem_alloc_option.allocator.free_func = wasm_free;

	/* initialize runtime environment */
	LOG_INF("Initializing WASM runtime...");

	if (!wasm_runtime_full_init(&init_args)) {
		LOG_ERR("Init runtime environment failed!");
		return 0;
	}

	bh_log_set_verbose_level(2);

	/* load WASM module */
	LOG_INF("Loading WASM module...");

	wasm_module_t wasm_module =
			wasm_runtime_load((uint8_t*) wasm_binary,
							  sizeof(wasm_binary),
							  error_buf,
							  sizeof(error_buf));
	if (!wasm_module) {
		LOG_ERR("Loading WASM module failed: %s", error_buf);
		goto fail1;
	}

	/* instantiate the module */
	LOG_INF("Instantiating WASM module...");

	wasm_module_inst_t wasm_module_inst =
			wasm_runtime_instantiate(wasm_module,
									 APP_STACK_SIZE,
									 APP_HEAP_SIZE,
									 error_buf,
									 sizeof(error_buf));

	if (!wasm_module_inst) {
		LOG_ERR("Instantiating WASM module failed: %s", error_buf);
		goto fail2;
	}

	/* invoke the main function */
	LOG_INF("Invoking main function...");
	wasm_application_execute_main(wasm_module_inst, 0, NULL);
	const char *exception = wasm_runtime_get_exception(wasm_module_inst);

	if (exception != NULL) {
		LOG_ERR("Exception occurred: %s", exception);
	}

	/* destroy the module instance */
	LOG_INF("Destroying WASM module instance...");
	wasm_runtime_deinstantiate(wasm_module_inst);

fail2:
	/* unload the module */
	LOG_INF("Unloading WASM module...");
	wasm_runtime_unload(wasm_module);

fail1:
	/* destroy runtime environment */
	LOG_INF("Destroying WASM runtime...");
	wasm_runtime_destroy();

	return 0;
}
