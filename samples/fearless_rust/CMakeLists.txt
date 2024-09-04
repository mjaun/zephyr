cmake_minimum_required(VERSION 3.20.0)
find_package(Zephyr REQUIRED HINTS $ENV{ZEPHYR_BASE})
project(fearless_rust)

include(cmake/ClangArgs.cmake)

set(RUST_BUILD_TYPE debug)
set(RUST_TARGET "riscv32i-unknown-none-elf")
set(RUST_TARGET_DIR ${CMAKE_CURRENT_BINARY_DIR}/rust/target)
set(RUST_LIBRARY "${RUST_TARGET_DIR}/${RUST_TARGET}/${RUST_BUILD_TYPE}/librustapp.a")
set(RUST_CONFIG ${CMAKE_CURRENT_SOURCE_DIR}/rust/.cargo/config.toml)

set(BINDGEN_CLANG_TARGET "riscv32-unknown-none-elf")
set(BINDGEN_CLANG_ARGS ${CMAKE_CURRENT_BINARY_DIR}/rust/clang_args.txt)
set(BINDGEN_WRAP_STATIC_FNS ${CMAKE_CURRENT_BINARY_DIR}/rust/static_fns.c)
set(BINDGEN_INPUT ${CMAKE_CURRENT_SOURCE_DIR}/src/bindgen_input.h)
set(BINDGEN_OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/rust/bindings.rs)

if(NOT CONFIG_RISCV_ISA_RV32I)
    message(FATAL_ERROR "Rust: Add support for other target")
endif()

configure_file(rust/config-template.toml ${RUST_CONFIG})
generate_clang_args(${BINDGEN_CLANG_ARGS} "--target=${BINDGEN_CLANG_TARGET}")

set(DUMMY_FILE ${CMAKE_CURRENT_BINARY_DIR}/rust/build-always)

add_custom_command(
        OUTPUT ${DUMMY_FILE}
        BYPRODUCTS ${RUST_LIBRARY}
        COMMAND cargo build
        COMMENT "Building Rust application"
        WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/rust
)

add_custom_target(rustapp ALL
        DEPENDS ${DUMMY_FILE}
)

add_dependencies(rustapp zephyr)
add_dependencies(app rustapp)

target_link_libraries(app PUBLIC ${RUST_LIBRARY})
target_sources(app PRIVATE src/main.c)
