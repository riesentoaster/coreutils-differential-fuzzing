use std::{
    fs::File,
    io::Write,
    process::{Command, Stdio},
};

use crate::executor::Executor;
static PATH: &str = "/dev/shm/temp/performance_test";

pub struct ConstantFileStdin;

impl Executor for ConstantFileStdin {
    fn execute(&mut self, _i: usize, path: &str) {
        let mut command = Command::new(path);

        command
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .stdin(File::open(PATH).unwrap());

        let child = command.spawn().expect("failed to start process");

        let output = child.wait_with_output().unwrap();
        assert!(!output.stdout.is_empty())
    }

    fn setup(&self) {
        File::create(PATH)
            .unwrap()
            .write_all("Hello, World!".as_bytes())
            .unwrap();
    }
}
