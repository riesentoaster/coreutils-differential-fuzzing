use libafl_bolts::shmem::{MmapShMemProvider, ShMem};

use crate::{setup_shmem, simple_stdin::simple_stdin, teardown_shmem, Executor};

pub struct ShmemPreload {
    size: usize,
    provider: MmapShMemProvider,
}

impl ShmemPreload {
    pub fn new(size: usize) -> Self {
        Self {
            size,
            provider: MmapShMemProvider::default(),
        }
    }
}

impl Executor for ShmemPreload {
    fn execute(&mut self, _i: usize, path: &str) {
        let shmem = setup_shmem(&mut self.provider, self.size);
        let description = shmem.description();
        let description_string = serde_json::to_string(&description).unwrap();
        let _res = simple_stdin(
            path,
            &[&description_string],
            &[("LD_PRELOAD", "target/release/libshmem_preload.so")],
            Some("Hello, World!"),
        );

        assert!(shmem.iter().all(|e| *e == 0x41));

        teardown_shmem(shmem);
    }
}
