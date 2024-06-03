use std::{io::Error as IOError, path::Path, process::Command};

use libafl::Error;
use libafl_bolts::shmem::{MmapShMem, MmapShMemProvider, ShMem, ShMemDescription, ShMemProvider};
use libc::{fcntl, FD_CLOEXEC, F_GETFD, F_SETFD};

pub fn get_coverage_shmem_size(util: String) -> Result<(usize, String), Error> {
    if !Path::new(&util).exists() {
        return Err(Error::illegal_argument(format!("Util {util} not found")));
    }

    let shared = "./target/release/libget_guard_num.so";
    if !Path::new(shared).exists() {
        return Err(Error::illegal_argument(
        "Missing shared library to instrument binary to find number of edges. Check Makefile.toml for the appropriate target."
        ));
    }

    let guard_num_command_output = Command::new(&util)
        .env("LD_PRELOAD", shared)
        .output()?
        .stdout;
    let guard_num = String::from_utf8(guard_num_command_output)?
        .trim()
        .parse::<usize>()?;

    match guard_num {
        0 => Err(Error::illegal_state("Binary reported a guard count of 0")),
        e => Ok((e, util)),
    }
}

fn make_shmem_persist(description: &ShMemDescription) -> Result<(), Error> {
    let fd = description.id.as_str().parse().unwrap();
    let flags = unsafe { fcntl(fd, F_GETFD) };

    if flags == -1 {
        return Err(Error::os_error(
            IOError::last_os_error(),
            "Failed to retrieve FD flags",
        ));
    }
    let result = unsafe { fcntl(fd, F_SETFD, flags & !FD_CLOEXEC) };
    if result == -1 {
        return Err(Error::os_error(
            IOError::last_os_error(),
            "Failed to set FD flags",
        ));
    }
    Ok(())
}

pub fn get_shmem(size: usize) -> Result<(MmapShMem, ShMemDescription), Error> {
    let mut shmem_provider = MmapShMemProvider::default();
    let shmem = shmem_provider
        .new_shmem(size)
        .expect("Could not get the shared memory map");

    let shmem_description = shmem.description();
    make_shmem_persist(&shmem_description)?;
    Ok((shmem, shmem_description))
}
