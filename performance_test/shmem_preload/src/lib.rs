#![allow(clippy::missing_safety_doc)]

use std::{
    ffi::CStr,
    mem::{size_of, transmute_copy},
};

use libafl_bolts::shmem::{MmapShMemProvider, ShMemDescription, ShMemProvider};

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
    extern "C" fn(),
    *mut c_void,
) -> i32;

static mut MAIN: Option<unsafe extern "C" fn(i32, *const *const u8, *const *const u8) -> i32> =
    None;

#[no_mangle]
#[allow(clippy::similar_names)]
pub unsafe extern "C" fn __libc_start_main(
    main: unsafe extern "C" fn(i32, *const *const u8, *const *const u8) -> i32,
    argc: i32,
    argv: *mut *const char,
    init: extern "C" fn(i32, *const *const u8, *const *const u8) -> i32,
    fini: extern "C" fn(),
    rtld_fini: extern "C" fn(),
    stack_end: *mut c_void,
) -> i32 {
    let orig_libc_start_main: LibcStartMainFunc = get_symbol(c"__libc_start_main", false);
    MAIN = Some(main);
    orig_libc_start_main(main_hook, argc, argv, init, fini, rtld_fini, stack_end)
}

#[no_mangle]
pub unsafe extern "C" fn main_hook(
    argc: i32,
    argv: *const *const u8,
    env: *const *const u8,
) -> i32 {
    let shmem_description_string =
        CStr::from_ptr(*argv.offset((argc - 1).try_into().unwrap()) as *const i8)
            .to_str()
            .expect("Could not parse shared memory description to string");

    let shmem_description: ShMemDescription = serde_json::from_str(shmem_description_string)
        .unwrap_or_else(|e| {
            panic!(
                "Could not parse shared memory description to struct \"{:?}\" — {:?}",
                shmem_description_string, e
            );
        });

    let mut shmem = MmapShMemProvider::default()
        .shmem_from_description(shmem_description)
        .expect("Could not acquire shared memory");

    shmem.fill(0x41);

    MAIN.unwrap()(argc - 1, argv, env)
}
