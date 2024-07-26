# SPDX-License-Identifier: Apache-2.0

# Rust make support

# Zephyr targets are defined through Kconfig.  We need to map these to
# an appropriate llvm target triple.  This sets `RUST_TARGET` in the
# parent scope, or an error if the target is not yet supported by
# Rust.
function(_rust_map_target)
  # Map Zephyr targets to LLVM targets.
  if(CONFIG_CPU_CORTEX_M)
    if(CONFIG_CPU_CORTEX_M0 OR CONFIG_CPU_CORTEX_M0PLUS OR CONFIG_CPU_CORTEX_M1)
      set(RUST_TARGET "thumbv6m-none-eabi" PARENT_SCOPE)
    elseif(CONFIG_CPU_CORTEX_M3)
      set(RUST_TARGET "thumbv7m-none-eabi" PARENT_SCOPE)
    elseif(CONFIG_CPU_CORTEX_M4)
      if(CONFIG_FP_HARDABI OR FORCE_FP_HARDABI)
        set(RUST_TARGET "thumbv7em-none-eabihf" PARENT_SCOPE)
      else()
        set(RUST_TARGET "thumbv7em-none-eabi" PARENT_SCOPE)
      endif()
    elseif(CONFIG_CPU_CORTEX_M23)
      set(RUST_TARGET "thumbv8m.base-none-eabi" PARENT_SCOPE)
    elseif(CONFIG_CPU_CORTEX_M33 OR CONFIG_CPU_CORTEX_M55)
      # Not a typo, Zephyr, uses ARMV7_M_ARMV8_M_FP to select the FP even on v8m.
      if(CONFIG_FP_HARDABI OR FORCE_FP_HARDABI)
        set(RUST_TARGET "thumbv8m.main-none-eabihf" PARENT_SCOPE)
      else()
        set(RUST_TARGET "thumbv8m.main-none-eabi" PARENT_SCOPE)
      endif()

      # Todo: The M55 is thumbv8.1m.main-none-eabi, which can be added when Rust
      # gain support for this target.
    else()
      message(FATAL_ERROR "Unknown Cortex-M target.")
    endif()
  elseif(CONFIG_RISCV)
    if(CONFIG_RISCV_ISA_RV64I)
      # TODO: Should fail if the extensions don't match.
      set(RUST_TARGET "riscv64imac-unknown-none-elf" PARENT_SCOPE)
    elseif(CONFIG_RISCV_ISA_RV32I)
      # TODO: We have multiple choices, try to pick the best.
      set(RUST_TARGET "riscv32i-unknown-none-elf" PARENT_SCOPE)
    else()
      message(FATAL_ERROR "Rust: Unsupported riscv ISA")
    endif()
  else()
    message(FATAL_ERROR "Rust: Add support for other target")
  endif()
endfunction()

# Gathers the compiler arguments for Clang which are used by bindgen to generate Rust FFI bindings.
function(_generate_clang_args BINDGEN_CLANG_ARGS)
  # Get compiler arguments from Zephyr
  zephyr_get_system_include_directories_for_lang(C system_includes)
  zephyr_get_include_directories_for_lang(C includes)
  zephyr_get_compile_definitions_for_lang(C definitions)

  # -imacros are needed but are part of zephyr_get_compile_options_for_lang() where many
  # things are not supported by Clang. Maybe there is a better way than hard coding.
  set(options "-imacros${AUTOCONF_H}")

  if(CONFIG_ENFORCE_ZEPHYR_STDINT)
    list(APPEND options "-imacros${ZEPHYR_BASE}/include/zephyr/toolchain/zephyr_stdint.h")
  endif()

  # Determine standard include directories of compiler.
  # I hope someone will figure out a nicer way of doing this.
  file(TOUCH ${CMAKE_CURRENT_BINARY_DIR}/empty.c)

  execute_process(
    COMMAND ${CMAKE_C_COMPILER} -E -Wp,-v ${CMAKE_CURRENT_BINARY_DIR}/empty.c
    OUTPUT_QUIET
    ERROR_VARIABLE output
    COMMAND_ERROR_IS_FATAL ANY
  )

  set(standard_includes "-nostdinc")
  if(output MATCHES "#include <\.\.\.> search starts here:\n(.*)\nEnd of search list\.")
    string(REGEX MATCHALL "[^ \n]+" paths "${CMAKE_MATCH_1}")
    foreach(path ${paths})
      get_filename_component(path ${path} ABSOLUTE)
      list(APPEND standard_includes "-isystem${path}")
    endforeach()
  else()
    message(WARNING "Unable to determine compiler standard include directories.")
  endif()

  # Not sure if a proper Clang target should be provided as well to generate the correct bindings.

  # Generate file containing arguments for Clang. Note that the file is generated after the
  # CMake configure stage as the variables contain generator expressions which cannot be
  # evaluated right now.
  file(
    GENERATE
    OUTPUT ${BINDGEN_CLANG_ARGS}
    CONTENT "${standard_includes};${system_includes};${includes};${definitions};${options}"
  )
endfunction()

function(rust_cargo_application)
  # For now, hard-code the Zephyr crate directly here.  Once we have
  # more than one crate, these should be added by the modules
  # themselves.
  set(LIB_RUST_CRATES zephyr zephyr-build zephyr-sys)

  _rust_map_target()
  message(STATUS "Building Rust llvm target ${RUST_TARGET}")

  # TODO: Make sure RUSTFLAGS is not set.

  # TODO: Let this be configurable, or based on Kconfig debug?
  set(RUST_BUILD_TYPE debug)
  set(BUILD_LIB_DIR "${CMAKE_CURRENT_SOURCE_DIR}/${RUST_TARGET}/${RUST_BUILD_TYPE}")

  set(CARGO_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust/target")
  set(RUST_LIBRARY "${CARGO_TARGET_DIR}/${RUST_TARGET}/${RUST_BUILD_TYPE}/librustapp.a")
  set(SAMPLE_CARGO_CONFIG "${CMAKE_CURRENT_BINARY_DIR}/rust/sample-cargo-config.toml")

  set(BINDGEN_CLANG_ARGS "${CMAKE_CURRENT_BINARY_DIR}/rust/clang_args.txt")
  set(BINDGEN_WRAP_STATIC_FNS "${CMAKE_CURRENT_BINARY_DIR}/rust/wrap_static_fns.c")

  file(MAKE_DIRECTORY "${CMAKE_CURRENT_BINARY_DIR}/rust")

  _generate_clang_args(${BINDGEN_CLANG_ARGS})

  # To get cmake to always invoke Cargo requires a bit of a trick.  We make the output of the
  # command a file that never gets created.  This will cause cmake to always rerun cargo.  We
  # add the actual library as a BYPRODUCTS list of this command, otherwise, the first time the
  # link will fail because it doesn't think it knows how to build the library.  This will also
  # cause the relink when the cargo command actually does rebuild the rust code.
  set(DUMMY_FILE "${CMAKE_BINARY_DIR}/always-run-cargo.dummy")

  # For each module in zephyr-rs, add entry both to the .cargo/config template and for the
  # command line, since either invocation will need to see these.
  set(command_paths)
  set(config_paths "")
  message(STATUS "Processing crates: ${ZEPHYR_RS_MODULES}")
  foreach(module IN LISTS LIB_RUST_CRATES)
    message(STATUS "module: ${module}")
    set(config_paths
      "${config_paths}\
${module}.path = \"${ZEPHYR_BASE}/lib/rust/${module}\"
")
    list(APPEND command_paths
      "--config"
      "patch.crates-io.${module}.path=\\\"${ZEPHYR_BASE}/lib/rust/${module}\\\""
      )
  endforeach()

  # Write out a cargo config file that can be copied into `.cargo/config.toml` (or made a
  # symlink) in the source directory to allow various IDE tools and such to work.  The build we
  # invoke will override these settings, in case they are out of date.  Everything set here
  # should match the arguments given to the cargo build command below.
  file(WRITE ${SAMPLE_CARGO_CONFIG} "
# This is a generated sample .cargo/config.toml file from the Zephyr build.
# At the time of generation, this represented the settings needed to allow
# a `cargo build` command to compile the rust code using the current Zephyr build.
# If any settings in the Zephyr build change, this could become out of date.
[build]
target = \"${RUST_TARGET}\"
target-dir = \"${CARGO_TARGET_DIR}\"

[env]
BUILD_DIR = \"${CMAKE_CURRENT_BINARY_DIR}\"
DOTCONFIG = \"${DOTCONFIG}\"
ZEPHYR_DTS = \"${ZEPHYR_DTS}\"
ZEPHYR_BASE = \"${ZEPHYR_BASE}\"
BINDGEN_CLANG_ARGS = \"${BINDGEN_CLANG_ARGS}\"
BINDGEN_WRAP_STATIC_FNS = \"${BINDGEN_WRAP_STATIC_FNS}\"

[patch.crates-io]
${config_paths}
")

  # The library is built by invoking Cargo.
  add_custom_command(
    OUTPUT ${DUMMY_FILE}
    BYPRODUCTS ${RUST_LIBRARY} ${BINDGEN_WRAP_STATIC_FNS}
    COMMAND
      ${CMAKE_EXECUTABLE}
      env BUILD_DIR=${CMAKE_CURRENT_BINARY_DIR}
      DOTCONFIG=${DOTCONFIG}
      ZEPHYR_DTS=${ZEPHYR_DTS}
      ZEPHYR_BASE=${ZEPHYR_BASE}
      BINDGEN_CLANG_ARGS=${BINDGEN_CLANG_ARGS}
      BINDGEN_WRAP_STATIC_FNS="${BINDGEN_WRAP_STATIC_FNS}"
      cargo build
      # TODO: release flag if release build
      # --release

      # Override the features according to the shield given. For a general case,
      # this will need to come from a variable or argument.
      # TODO: This needs to be passed in.
      # --no-default-features
      # --features ${SHIELD_FEATURE}

      # Set a replacement so that packages can just use `zephyr-sys` as a package
      # name to find it.
      ${command_paths}
      --target ${RUST_TARGET}
      --target-dir ${CARGO_TARGET_DIR}
    COMMENT "Building Rust application"
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
  )

  add_custom_target(librustapp ALL
    DEPENDS ${DUMMY_FILE}
  )

  target_link_libraries(app PUBLIC -Wl,--allow-multiple-definition ${RUST_LIBRARY})
  add_dependencies(app librustapp)

  # Not sure if this file belongs to the zephyr or the app target
  zephyr_sources(${BINDGEN_WRAP_STATIC_FNS})

  # Typically this property would be set via zephyr_library_app_memory(), but since we don't have a Zephyr library
  # here we set it manually to move any globals from our Rust library to a dedicated partition.
  # This is useful when using CONFIG_USERSPACE to enable access to globals when running threads in user mode.
  get_filename_component(RUST_LIBRARY_NAME ${RUST_LIBRARY} NAME)
  set_property(TARGET zephyr_property_target APPEND PROPERTY COMPILE_OPTIONS "-l" "${RUST_LIBRARY_NAME}" "rust_mem_part")

  # Presumably, Rust applications will have no C source files, but cmake will require them.
  # Add an empty file so that this will build.  The main will come from the rust library.
  target_sources(app PRIVATE ${ZEPHYR_BASE}/lib/rust/main.c)
endfunction()
