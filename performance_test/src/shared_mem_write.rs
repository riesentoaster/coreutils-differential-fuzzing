use libafl_bolts::shmem::MmapShMemProvider;

use crate::{setup_shmem, teardown_shmem, Executor};

pub struct SharedMemWrite {
    p: MmapShMemProvider,
    size: usize,
}

impl SharedMemWrite {
    pub fn new(size: usize) -> Self {
        Self {
            p: MmapShMemProvider::default(),
            size,
        }
    }
}

impl Executor for SharedMemWrite {
    fn execute(&mut self, i: usize, _path: &str) {
        let fill = i as u8;
        let mut shmem = setup_shmem(&mut self.p, self.size);
        shmem.fill(fill);
        assert!(shmem.iter().all(|e| *e == fill));
        teardown_shmem(shmem);
    }
}
