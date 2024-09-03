cmake_minimum_required(VERSION 3.20.0)
find_package(Zephyr REQUIRED HINTS $ENV{ZEPHYR_BASE})
project(fearless_rust)

target_sources(app PRIVATE
        src/main.c
        src/wifi.c
        src/udp.c
        src/measurement.c
        src/serializer.c
        src/device.c
)
