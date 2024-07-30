// Copyright (c) 2024 Zühlke Engineering AG
// SPDX-License-Identifier: Apache-2.0

use core::alloc::{GlobalAlloc, Layout};
use core::ffi::{c_int, c_uint, c_void};
use crate::errno::{check_ptr_mut, ENOMEM, ErrnoResult};

pub(crate) trait KernelObject {
    fn get_type_id() -> c_uint;
}

impl KernelObject for zephyr_sys::k_thread {
    fn get_type_id() -> c_uint {
        zephyr_sys::k_objects_K_OBJ_THREAD
    }
}

impl KernelObject for zephyr_sys::k_mutex {
    fn get_type_id() -> c_uint {
        zephyr_sys::k_objects_K_OBJ_MUTEX
    }
}

pub(crate) unsafe fn object_alloc<T: KernelObject>() -> ErrnoResult<*mut T> {
    check_ptr_mut(zephyr_sys::k_object_alloc(T::get_type_id()) as *mut T, ENOMEM)
}

pub(crate) unsafe fn object_free<T: KernelObject>(obj: *mut T) {
    zephyr_sys::k_object_free(obj as *mut c_void);
}

pub(crate) unsafe fn thread_stack_alloc(size: usize, flags: u32) -> ErrnoResult<*mut zephyr_sys::k_thread_stack_t> {
    check_ptr_mut(zephyr_sys::k_thread_stack_alloc(size, flags as c_int), ENOMEM)
}

extern "C" {
    static mut rust_heap: zephyr_sys::k_heap;
}

struct ZephyrAllocator {}

#[global_allocator]
static ALLOCATOR: ZephyrAllocator = ZephyrAllocator {};

unsafe impl GlobalAlloc for ZephyrAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        #![allow(static_mut_refs)]
        zephyr_sys::k_heap_aligned_alloc(
            &mut rust_heap, layout.align(),
            layout.size(),
            zephyr_sys::k_timeout_t { ticks: 0 },
        ) as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        #![allow(static_mut_refs)]
        zephyr_sys::k_heap_free(&mut rust_heap, ptr as *mut c_void)
    }
}