use std::{
    fs::{remove_file, File},
    io::Write,
};

use crate::executor::Executor;
static PARENT: &str = "/dev/shm/temp/file_write_only";

pub struct FileWriteOnly<'a> {
    arr: &'a [u8],
}

impl<'a> FileWriteOnly<'a> {
    pub fn new(arr: &'a [u8]) -> Self {
        Self { arr }
    }
}

impl<'a> Executor for FileWriteOnly<'a> {
    fn execute(&mut self, i: usize, _path: &str) {
        let path = format!("{}{}", PARENT, i);
        File::create(&path).unwrap().write_all(self.arr).unwrap();
        remove_file(path).unwrap();
    }
}
