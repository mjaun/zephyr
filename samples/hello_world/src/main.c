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
#include "test_wasm.h"

#include <zephyr/logging/log.h>
LOG_MODULE_REGISTER(main);

#define GLOBAL_HEAP_BUF_SIZE 131072
#define APP_STACK_SIZE 8192
#define APP_HEAP_SIZE 8192

#ifdef CONFIG_NO_OPTIMIZATIONS
#define MAIN_THREAD_STACK_SIZE 8192
#else
#define MAIN_THREAD_STACK_SIZE 4096
#endif

#define MAIN_THREAD_PRIORITY 5

static int app_argc;
static char **app_argv;

static char global_heap_buf[GLOBAL_HEAP_BUF_SIZE] = {0};

void iwasm_main() {
	char error_buf[128];

	RuntimeInitArgs init_args;
	memset(&init_args, 0, sizeof(RuntimeInitArgs));
	init_args.mem_alloc_type = Alloc_With_Pool;
	init_args.mem_alloc_option.pool.heap_buf = global_heap_buf;
	init_args.mem_alloc_option.pool.heap_size = sizeof(global_heap_buf);

	/* initialize runtime environment */
	LOG_INF("Initializing WASM runtime...");

	if (!wasm_runtime_full_init(&init_args)) {
		LOG_ERR("Init runtime environment failed!");
		return;
	}

	bh_log_set_verbose_level(2);

	/* load WASM byte buffer from byte buffer of include file */
	uint8 *wasm_file_buf = (uint8 *) wasm_test_file;
	uint32 wasm_file_size = sizeof(wasm_test_file);

	/* load WASM module */
	LOG_INF("Loading WASM module...");

	wasm_module_t wasm_module =
			wasm_runtime_load(wasm_file_buf,
							  wasm_file_size,
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
	wasm_application_execute_main(wasm_module_inst, app_argc, app_argv);
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
}

K_THREAD_STACK_DEFINE(iwasm_main_thread_stack, MAIN_THREAD_STACK_SIZE);
static struct k_thread iwasm_main_thread;

static bool iwasm_init(void) {
	k_tid_t tid = k_thread_create(&iwasm_main_thread,
								  iwasm_main_thread_stack,
								  MAIN_THREAD_STACK_SIZE,
								  iwasm_main,
								  NULL, NULL, NULL,
								  MAIN_THREAD_PRIORITY,
								  0, K_NO_WAIT);
	return tid ? true : false;
}

int main(void) {
	iwasm_init();
	return 0;
}
