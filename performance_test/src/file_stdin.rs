use std::{
    fs::{create_dir_all, remove_file, File},
    io::Write,
    process::{Command, Stdio},
};

use crate::executor::Executor;
static PARENT: &str = "/dev/shm/temp/file_write_only";

pub struct FileStdin<'a> {
    arr: &'a [u8],
}

impl<'a> FileStdin<'a> {
    pub fn new(arr: &'a [u8]) -> Self {
        Self { arr }
    }
}

impl<'a> Executor for FileStdin<'a> {
    fn execute(&mut self, i: usize, exec_path: &str) {
        let path = format!("{}{}", PARENT, i);
        File::create(&path).unwrap().write_all(self.arr).unwrap();
        let mut command = Command::new(exec_path);

        command
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .stdin(File::open(&path).unwrap());

        let child = command.spawn().expect("failed to start process");

        let output = child.wait_with_output().unwrap();
        assert!(!output.stdout.is_empty());
        remove_file(path).unwrap();
    }

    fn setup(&self) {
        create_dir_all("/dev/shm/temp/").unwrap();
    }
}
