#![allow(clippy::missing_safety_doc)]

use std::{
    ffi::CStr,
    mem::{size_of, transmute_copy},
};

use libc::{c_void, dlerror, dlsym, RTLD_DEFAULT, RTLD_NEXT};

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
    unsafe fn(),
    *mut c_void,
) -> i32;

static mut RTLD_FINI: Option<extern "C" fn()> = None;

#[no_mangle]
unsafe fn custom_rtld_fini() {
    let mut res = 0;
    for i in 0..10 {
        res += i
    }
    assert_eq!(45, res);
    RTLD_FINI.unwrap()()
}

#[no_mangle]
#[allow(clippy::similar_names)]
pub unsafe extern "C" fn __libc_start_main(
    main: unsafe extern "C" fn(i32, *const *const u8, *const *const u8) -> i32,
    argc: i32,
    argv: *const *const char,
    init: extern "C" fn(i32, *const *const u8, *const *const u8) -> i32,
    fini: extern "C" fn(),
    rtld_fini: extern "C" fn(),
    stack_end: *mut c_void,
) -> i32 {
    RTLD_FINI = Some(rtld_fini);
    let orig_libc_start_main: LibcStartMainFunc = get_symbol(c"__libc_start_main", false);
    orig_libc_start_main(main, argc, argv, init, fini, custom_rtld_fini, stack_end)
}
