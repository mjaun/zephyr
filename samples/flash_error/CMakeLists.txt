cmake_minimum_required(VERSION 3.20.0)

set(BOARD nucleo_h743zi_minimal)

find_package(Zephyr REQUIRED HINTS $ENV{ZEPHYR_BASE})
project(flash_error)

target_sources(app PRIVATE src/main.c)
