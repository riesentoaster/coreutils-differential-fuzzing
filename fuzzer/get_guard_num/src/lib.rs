#![allow(clippy::missing_safety_doc)]

use libc::{c_void, dlerror, dlsym, RTLD_DEFAULT, RTLD_NEXT};

use std::{
    ffi::CStr,
    mem::{size_of, transmute_copy},
};

pub unsafe fn get_symbol<T>(name: &CStr, search_global: bool) -> T {
    assert_eq!(
        size_of::<*mut c_void>(),
        size_of::<T>(),
        "T must be the same size as a pointer."
    );

    let handle = if search_global {
        RTLD_DEFAULT
    } else {
        RTLD_NEXT
    };

    let symbol_pointer: *mut c_void = dlsym(handle, name.as_ptr());
    if symbol_pointer.is_null() {
        panic!(
            "Got a NULL pointer, could not load symbol {:#?}: {:#?}",
            name,
            dlerror()
        );
    }
    transmute_copy(&symbol_pointer)
}

pub type LibcStartMainFunc = fn(
    unsafe extern "C" fn(i32, *const *const u8, *const *const u8) -> i32,
    i32,
    *const *const char,
    extern "C" fn(i32, *const *const u8, *const *const u8) -> i32,
    extern "C" fn(),
    unsafe extern "C" fn(),
    *mut c_void,
) -> i32;

#[no_mangle]
#[allow(clippy::similar_names)]
pub unsafe extern "C" fn __libc_start_main(
    _main: unsafe extern "C" fn(i32, *const *const u8, *const *const u8) -> i32,
    argc: i32,
    argv: *mut *const char,
    init: extern "C" fn(i32, *const *const u8, *const *const u8) -> i32,
    fini: extern "C" fn(),
    rtld_fini: unsafe extern "C" fn(),
    stack_end: *mut c_void,
) -> i32 {
    let orig_libc_start_main: LibcStartMainFunc = get_symbol(c"__libc_start_main", false);
    orig_libc_start_main(main_hook, argc, argv, init, fini, rtld_fini, stack_end)
}

#[no_mangle]
pub unsafe extern "C" fn main_hook(
    _argc: i32,
    _argv: *const *const u8,
    _env: *const *const u8,
) -> i32 {
    let get_guard_count: fn() -> usize = get_symbol(c"get_guard_count", true);
    let guard_count = get_guard_count();
    println!("{}", guard_count);
    0
}
