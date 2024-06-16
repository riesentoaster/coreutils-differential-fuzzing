use std::{
    io::Write,
    process::{Command, Stdio},
};

use crate::executor::Executor;

pub struct SimpleStdin;

impl Executor for SimpleStdin {
    fn execute(&mut self, _i: usize, path: &str) {
        assert!(!simple_stdin(path, &[], &[], Some("Hello, World!")).is_empty());
    }
}

#[must_use]
pub fn simple_stdin(
    path: &str,
    args: &[&str],
    env: &[(&str, &str)],
    stdin_content: Option<&str>,
) -> Vec<u8> {
    let mut command = Command::new(path);

    env.iter().for_each(|(key, val)| {
        command.env(key, val);
    });

    command.args(args);

    if stdin_content.is_some() {
        command
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .stdin(Stdio::piped());
    } else {
        command.stderr(Stdio::null()).stdout(Stdio::null());
    }

    let mut child = command.spawn().expect("failed to start process");
    if let Some(stdin_content) = stdin_content {
        if let Some(mut stdin) = child.stdin.take() {
            writeln!(stdin, "{}", stdin_content).expect("Failed to write to stdin");
        }
    }

    child.wait_with_output().unwrap().stdout
}
