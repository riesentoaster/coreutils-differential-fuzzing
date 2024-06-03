use std::{
    borrow::Cow,
    process::{Command, Stdio},
};

use libafl::{
    corpus::Testcase, events::EventFirer, executors::ExitKind, feedbacks::Feedback,
    observers::ObserversTuple, state::State, Error,
};
use libafl_bolts::Named;

use super::executor::{pseudo_pipe, ExtractsToCommand};

pub struct CovFeedback {
    is_interesting: bool,
    gcov_path: String,
    temp_file_stdin_path: String,
}

impl CovFeedback {
    pub fn new(is_interesting: bool, gcov_path: String, temp_file_stdin_path: String) -> Self {
        Self {
            is_interesting,
            gcov_path,
            temp_file_stdin_path: format!("/dev/shm/temp{}", temp_file_stdin_path),
        }
    }
}

impl<S> Feedback<S> for CovFeedback
where
    S: State,
    S::Input: ExtractsToCommand,
{
    fn is_interesting<EM, OT>(
        &mut self,
        _state: &mut S,
        _manager: &mut EM,
        _input: &<S>::Input,
        _observers: &OT,
        _exit_kind: &ExitKind,
    ) -> Result<bool, Error>
    where
        EM: EventFirer<State = S>,
        OT: ObserversTuple<S>,
    {
        Ok(self.is_interesting)
    }

    fn append_metadata<EM, OT>(
        &mut self,
        _state: &mut S,
        _manager: &mut EM,
        _observers: &OT,
        testcase: &mut Testcase<<S>::Input>,
    ) -> Result<(), Error>
    where
        OT: ObserversTuple<S>,
        EM: EventFirer<State = S>,
    {
        let input = testcase
            .input()
            .as_ref()
            .ok_or(Error::illegal_state("Should have an input at this point"))?;
        Command::new(&self.gcov_path)
            .args(input.get_args())
            .stdin(pseudo_pipe(input.get_stdin(), &self.temp_file_stdin_path)?)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?
            .wait()?;
        Ok(())
    }
}

impl Named for CovFeedback {
    fn name(&self) -> &Cow<'static, str> {
        &Cow::Borrowed("CovFeedback")
    }
}
