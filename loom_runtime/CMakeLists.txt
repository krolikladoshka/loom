cmake_minimum_required(VERSION 3.31)
project(loom_runtime C ASM)


add_compile_options(-fno-stack-check)

set(CMAKE_C_STANDARD 11)


find_package(PkgConfig REQUIRED)
find_package(Threads REQUIRED)

add_executable(loom_runtime
        main.c
        asm/restore_context.asm
        loom_memory.c
        loom_scheduler.h
        loom_common.h
        loom_memory.h
        loom_scheduler.c
        loom_runtime.c
        utils.h
)
target_compile_options(loom_runtime PRIVATE
        -arch arm64
        -mbranch-protection=none
        -fno-stack-check
        -fno-stack-protector
)
target_link_options(loom_runtime PRIVATE
        -Wl,-no_pie
)