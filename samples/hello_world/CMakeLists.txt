# SPDX-License-Identifier: Apache-2.0

cmake_minimum_required(VERSION 3.20.0)

find_package(Zephyr REQUIRED HINTS $ENV{ZEPHYR_BASE})
project(hello_world)

set(gen_dir ${ZEPHYR_BINARY_DIR}/include/generated/)
#generate_inc_file_for_target(app apps/hello_world_c/test.wasm ${gen_dir}/test.wasm.inc)
generate_inc_file_for_target(app apps/hello_world_rs/test.wasm ${gen_dir}/test.wasm.inc)

target_sources(app PRIVATE src/main.c)
