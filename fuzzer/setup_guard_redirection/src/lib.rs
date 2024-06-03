#![allow(clippy::missing_safety_doc)]

use core::slice;
use std::{
    ffi::CStr,
    fmt::Debug,
    fs::OpenOptions,
    io::Write,
    mem::{size_of, transmute_copy},
    panic::{catch_unwind, AssertUnwindSafe},
    ptr::null,
    time::{SystemTime, UNIX_EPOCH},
};

use libafl_bolts::{
    shmem::{MmapShMemProvider, ShMemDescription, ShMemProvider},
    AsSliceMut,
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

static mut SHMEM_DESCRIPTION: Option<ShMemDescription> = None;
static mut RTLD_FINI: Option<extern "C" fn()> = None;

unsafe fn extract_shmem_description(argc: &mut i32, argv: *mut *const char) {
    assert!(*argc > 1, "Did not receive shared memory description");
    let last_arg = argv.offset((*argc - 1).try_into().unwrap());
    assert!(
        !last_arg.is_null(),
        "Passed a string of length 0 to the wrapper, expecting the shared memory description"
    );

    let shmem_description_string = CStr::from_ptr(*last_arg as *const i8)
        .to_str()
        .expect("Could not parse shared memory description to string");

    SHMEM_DESCRIPTION = Some(
        serde_json::from_str(shmem_description_string).unwrap_or_else(|e| {
            panic!(
                "Could not parse shared memory description to struct \"{:?}\" — {:?}",
                shmem_description_string, e
            )
        }),
    );

    // making the shmem passing transparent to the binary under test
    *argc -= 1;
    *last_arg = null();
}

fn log<T: Debug>(s: T) {
    OpenOptions::new()
        .append(true)
        .create(true)
        .open("redirection.log")
        .expect("Failed to open file")
        .write_all(
            format!(
                "{}: {:?}\n",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis(),
                s
            )
            .as_bytes(),
        )
        .expect("Failed to write to file")
}

#[no_mangle]
unsafe fn write_guards() {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let shmem_description = SHMEM_DESCRIPTION.expect("No shared memory descriptor was saved");
        let mut shmem = MmapShMemProvider::default()
            .shmem_from_description(shmem_description)
            .expect("Could not acquire shared memory");
        let get_guard_count: fn() -> usize = get_symbol(c"get_guard_count", true);
        let guard_count = get_guard_count();
        let shmem_len = shmem.len();
        if shmem_len != guard_count {
            log(format!(
                "Memory sizes don't match. shmem: {}, guards: {}",
                shmem_len, guard_count
            ));
        } else {
            let shmem_target = shmem.as_slice_mut();

            let get_guard_values: fn() -> *const i32 = get_symbol(c"get_guard_values", true);
            let guards = get_guard_values();
            let guard_slice = slice::from_raw_parts(guards, guard_count);
            shmem_target
                .copy_from_slice(&guard_slice.iter().map(|&x| x as u8).collect::<Vec<u8>>());
        }
    }));

    if let Err(e) = result {
        log(if let Some(msg) = e.downcast_ref::<&'static str>() {
            msg.to_string()
        } else if let Some(msg) = e.downcast_ref::<String>() {
            msg.clone()
        } else {
            "Panic occurred but the message is not a string.".to_string()
        })
    }

    RTLD_FINI.expect("Did not previously store a reference to the original rtld_fini function")();
}

#[no_mangle]
#[allow(clippy::similar_names)]
pub unsafe extern "C" fn __libc_start_main(
    main: unsafe extern "C" fn(i32, *const *const u8, *const *const u8) -> i32,
    mut argc: i32,
    argv: *mut *const char,
    init: extern "C" fn(i32, *const *const u8, *const *const u8) -> i32,
    fini: extern "C" fn(),
    rtld_fini: extern "C" fn(),
    stack_end: *mut c_void,
) -> i32 {
    extract_shmem_description(&mut argc, argv);
    RTLD_FINI = Some(rtld_fini);
    let orig_libc_start_main: LibcStartMainFunc = get_symbol(c"__libc_start_main", false);
    orig_libc_start_main(main, argc, argv, init, fini, write_guards, stack_end)
}
