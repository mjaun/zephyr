cmake_minimum_required(VERSION 3.20.0)
find_package(Zephyr REQUIRED HINTS $ENV{ZEPHYR_BASE})
project(fearless_rust)

set(RUST_SOURCE_DIR ${CMAKE_CURRENT_SOURCE_DIR}/rust)
set(RUST_BINARY_DIR ${CMAKE_CURRENT_BINARY_DIR}/rust)
set(RUST_BUILD_TYPE debug)
set(RUST_TARGET "riscv32i-unknown-none-elf")
set(RUST_TARGET_DIR ${RUST_BINARY_DIR}/target)
set(RUST_LIBRARY "${RUST_TARGET_DIR}/${RUST_TARGET}/${RUST_BUILD_TYPE}/librustapp.a")
set(RUST_CONFIG ${RUST_SOURCE_DIR}/.cargo/config.toml)

if(NOT CONFIG_RISCV_ISA_RV32I)
    message(FATAL_ERROR "Rust: Add support for other target")
endif()

configure_file(rust/config-template.toml ${RUST_CONFIG})

set(DUMMY_FILE ${RUST_BINARY_DIR}/build-always)

add_custom_command(
        OUTPUT ${DUMMY_FILE}
        BYPRODUCTS ${RUST_LIBRARY}
        COMMAND cargo build
        COMMENT "Building Rust application"
        WORKING_DIRECTORY ${RUST_SOURCE_DIR}
)

add_custom_target(rustapp ALL
        DEPENDS ${DUMMY_FILE}
)

add_dependencies(rustapp zephyr)
add_dependencies(app rustapp)

target_link_libraries(app PUBLIC ${RUST_LIBRARY})
target_sources(app PRIVATE src/main.c)
