use std::{path::Path, process::Command};

use libafl::Error;
use libafl_bolts::shmem::{MmapShMem, MmapShMemProvider, ShMem, ShMemDescription, ShMemProvider};

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

pub fn get_shmem(size: usize) -> Result<(MmapShMem, ShMemDescription), Error> {
    let mut shmem_provider = MmapShMemProvider::default();
    let shmem = shmem_provider
        .new_shmem(size)
        .expect("Could not get the shared memory map");

    shmem.persist_for_child_processes()?;

    let shmem_description = shmem.description();
    Ok((shmem, shmem_description))
}
