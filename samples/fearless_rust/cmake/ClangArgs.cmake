function(generate_clang_args BINDGEN_CLANG_ARGS)
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
    # I hope someone knows a nicer way of doing this.
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

    # Not sure if a proper target should be provided as well to generate the correct bindings.

    # Generate file containing arguments for Clang. Note that the file is generated after the
    # CMake configure stage as the variables contain generator expressions which cannot be
    # evaluated right now.
    file(
            GENERATE
            OUTPUT ${BINDGEN_CLANG_ARGS}
            CONTENT "${standard_includes};${system_includes};${includes};${definitions};${options};${ARGN}"
    )
endfunction()
