use std::{
    borrow::Cow,
    ffi::OsStr,
    fs::File,
    io::Write,
    marker::PhantomData,
    process::{Child, Command, Stdio},
    time::Duration,
};

use libafl::{
    executors::{command::CommandConfigurator, CommandExecutor},
    observers::{StdErrObserver, StdOutObserver},
    state::State,
    Error,
};
use libafl_bolts::{
    shmem::ShMemDescription,
    tuples::{Handle, MatchName},
};
use serde::Serialize;

// Create the executor for an in-process function with just one observer
#[derive(Debug)]
pub struct CoverageCommandExecutor<I: ExtractsToCommand> {
    shmem_coverage_description: String,
    temp_file_stdin_path: String,
    stdout_observer: Option<Handle<StdOutObserver>>,
    stderr_observer: Option<Handle<StdErrObserver>>,
    util: String,
    phantom: PhantomData<I>,
}

impl<I: ExtractsToCommand> CoverageCommandExecutor<I> {
    pub fn new<OT, S, ID>(
        shmem_coverage_description: &ShMemDescription,
        stdout_observer: Option<Handle<StdOutObserver>>,
        stderr_observer: Option<Handle<StdErrObserver>>,
        observers: OT,
        util: &str,
        id: ID,
    ) -> CommandExecutor<OT, S, CoverageCommandExecutor<I>>
    where
        S: State,
        S::Input: ExtractsToCommand,
        OT: MatchName,
        ID: ToString,
    {
        let serialized_description = serde_json::to_string(&shmem_coverage_description)
            .expect("Could not stringify shared memory description");

        let configurator = Self {
            shmem_coverage_description: serialized_description,
            temp_file_stdin_path: format!("/dev/shm/temp{}", id.to_string()),
            stdout_observer,
            stderr_observer,
            util: util.to_string(),
            phantom: PhantomData,
        };
        configurator.into_executor(observers)
    }
}

pub trait ExtractsToCommand: Serialize {
    fn get_stdin(&self) -> &Vec<u8>;
    fn get_args<'a>(&self) -> Vec<Cow<'a, OsStr>>;
}

impl<I> CommandConfigurator<I> for CoverageCommandExecutor<I>
where
    I: ExtractsToCommand,
{
    fn spawn_child(&mut self, input: &I) -> Result<Child, Error> {
        let mut command = Command::new(&self.util);

        command
            .env(
                "LD_PRELOAD",
                "./target/release/libsetup_guard_redirection.so",
            )
            .args(input.get_args())
            .arg(&self.shmem_coverage_description)
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .stdin(pseudo_pipe(input.get_stdin(), &self.temp_file_stdin_path)?);

        let child = command.spawn().expect("failed to start process");
        Ok(child)
    }

    fn exec_timeout(&self) -> Duration {
        Duration::from_secs(30)
    }

    fn stdout_observer(&self) -> Option<Handle<StdOutObserver>> {
        self.stdout_observer.clone()
    }

    fn stderr_observer(&self) -> Option<Handle<StdErrObserver>> {
        self.stderr_observer.clone()
    }
}

/// Creates a [`File`] that can be used to write data to a [`Command`]'s `stdin`.
///
/// The implementation relies on a temp file on disk. Consider using an in-memory file, e.g. by locating it in `/dev/shm/`.
///
/// # Errors on
///
/// This function will return an error if the underlying os functions error.
pub fn pseudo_pipe(data: &[u8], path: &str) -> Result<File, Error> {
    File::create(path)
        .map_err(|e| Error::os_error(e, "Could not create temp file"))?
        .write_all(data)
        .map_err(|e| Error::os_error(e, "Could not write data to temp file"))?;
    File::open(path).map_err(|e| Error::os_error(e, "Could not open temp file again"))
}
