use std::ffi::CStr;

use libafl_bolts::shmem::{MmapShMem, MmapShMemProvider, ShMem, ShMemDescription, ShMemProvider};
use libc::{close, fcntl, FD_CLOEXEC, F_GETFD, F_SETFD};
fn make_shmem_persist(description: &ShMemDescription) {
    let fd = description.id.as_str().parse().unwrap();
    let flags = unsafe { fcntl(fd, F_GETFD) };

    if flags == -1 {
        panic!("Failed to retrieve FD flags",);
    }
    let result = unsafe { fcntl(fd, F_SETFD, flags & !FD_CLOEXEC) };
    if result == -1 {
        panic!("Failed to set FD flags",);
    }
}
fn unmake_shmem_persist(description: &ShMemDescription) {
    let fd = description.id.as_str().parse().unwrap();
    let flags = unsafe { fcntl(fd, F_GETFD) };

    if flags == -1 {
        panic!("Failed to retrieve FD flags",);
    }
    let result = unsafe { fcntl(fd, F_SETFD, flags & FD_CLOEXEC) };
    if result == -1 {
        panic!("Failed to set FD flags",);
    }
}

pub fn setup_shmem(provider: &mut MmapShMemProvider, size: usize) -> MmapShMem {
    let shmem = provider.new_shmem(size).unwrap();
    make_shmem_persist(&shmem.description());
    shmem
}

pub fn teardown_shmem(shmem: MmapShMem) {
    unmake_shmem_persist(&shmem.description());
    let fd = CStr::from_bytes_until_nul(shmem.id().as_array())
        .unwrap()
        .to_str()
        .unwrap()
        .parse()
        .unwrap();
    unsafe { close(fd) };
    drop(shmem);
}
