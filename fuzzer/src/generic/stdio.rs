use std::{borrow::Cow, fmt::Display};

use std::fmt::Write;

use libafl::{
    events::EventFirer,
    executors::ExitKind,
    feedbacks::Feedback,
    observers::{ObserversTuple, StdErrObserver, StdOutObserver},
    state::State,
    Error, HasMetadata, SerdeAny,
};
use libafl_bolts::{
    tuples::{Handle, MatchNameRef},
    Named,
};

use serde::{Deserialize, Serialize};

#[cfg(feature = "differential")]
use libafl_bolts::tuples::Handled;

#[derive(Clone)]
pub struct DiffStdIOMetadataPseudoFeedback {
    name1: Cow<'static, str>,
    name2: Cow<'static, str>,
    stderr_observer1: Handle<StdErrObserver>,
    stderr_observer2: Handle<StdErrObserver>,
    stdout_observer1: Handle<StdOutObserver>,
    stdout_observer2: Handle<StdOutObserver>,
    exit_kind: Option<ExitKind>,
}

impl DiffStdIOMetadataPseudoFeedback {
    #[cfg(feature = "differential")]
    pub fn new(
        name1: &str,
        name2: &str,
        stderr_observer1: &StdErrObserver,
        stderr_observer2: &StdErrObserver,
        stdout_observer1: &StdOutObserver,
        stdout_observer2: &StdOutObserver,
    ) -> Self {
        Self {
            name1: Cow::Owned(name1.to_string()),
            name2: Cow::Owned(name2.to_string()),
            stderr_observer1: stderr_observer1.handle(),
            stderr_observer2: stderr_observer2.handle(),
            stdout_observer1: stdout_observer1.handle(),
            stdout_observer2: stdout_observer2.handle(),
            exit_kind: None,
        }
    }
}

impl<S> Feedback<S> for DiffStdIOMetadataPseudoFeedback
where
    S: State,
    S::Input: Display,
{
    fn is_interesting<EM, OT>(
        &mut self,
        _state: &mut S,
        _manager: &mut EM,
        _input: &S::Input,
        _observers: &OT,
        exit_kind: &ExitKind,
    ) -> Result<bool, Error>
    where
        EM: EventFirer<State = S>,
        OT: ObserversTuple<S>,
    {
        self.exit_kind = Some(*exit_kind);
        Ok(false)
    }

    fn append_metadata<EM, OT>(
        &mut self,
        _state: &mut S,
        _manager: &mut EM,
        observers: &OT,
        testcase: &mut libafl::prelude::Testcase<<S>::Input>,
    ) -> Result<(), Error>
    where
        OT: ObserversTuple<S>,
        EM: EventFirer<State = S>,
    {
        fn f<'a, T, OT>(handle: &Handle<T>, observers: &'a OT) -> Result<&'a T, Error>
        where
            OT: MatchNameRef,
        {
            match observers.get(handle) {
                None => Err(Error::illegal_argument(format!(
                    "DiffFeedback: observer {} not found",
                    handle.name()
                ))),
                Some(e) => Ok(e),
            }
        }

        let input = testcase.input().as_ref().map(|e| e.to_string());

        let exit_kind_string = self
            .exit_kind
            .map_or("No ExitKind recorded".to_string(), |e| format!("{:?}", e));

        testcase
            .metadata_map_mut()
            .insert(DiffStdIOMetadataPseudoFeedbackMetadata {
                input,
                name1: self.name1.to_string(),
                name2: self.name2.to_string(),
                stderr_observer1: vec_string_mapper(&f(&self.stderr_observer1, observers)?.stderr),
                stderr_observer2: vec_string_mapper(&f(&self.stderr_observer2, observers)?.stderr),
                stdout_observer1: vec_string_mapper(&f(&self.stdout_observer1, observers)?.stdout),
                stdout_observer2: vec_string_mapper(&f(&self.stdout_observer2, observers)?.stdout),
                exit_kind: exit_kind_string,
            });
        Ok(())
    }
}

impl Named for DiffStdIOMetadataPseudoFeedback {
    fn name(&self) -> &Cow<'static, str> {
        &Cow::Borrowed("DiffStdioMetadataPseudoFeedback")
    }
}

#[derive(Debug, SerdeAny, Serialize, Deserialize)]
struct DiffStdIOMetadataPseudoFeedbackMetadata {
    input: Option<String>,
    name1: String,
    name2: String,
    exit_kind: String,
    stderr_observer1: String,
    stderr_observer2: String,
    stdout_observer1: String,
    stdout_observer2: String,
}

pub fn vec_string_mapper(v: &Option<Vec<u8>>) -> String {
    v.as_ref()
        .map(|v| {
            std::str::from_utf8(v.as_ref()).map_or(
                format!(
                    "utf8 error, lossy string: '{}', bytes: 0x{}",
                    String::from_utf8_lossy(v),
                    v.iter()
                        .fold(String::with_capacity(v.len() * 2), |mut w, byte| {
                            write!(w, "{:02x}", byte).unwrap();
                            w
                        })
                ),
                |s| s.to_string(),
            )
        })
        .unwrap_or("Did not observe anything".to_string())
}
