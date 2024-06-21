use std::borrow::Cow;

use libafl::{
    events::EventFirer,
    executors::{DiffExitKind, ExitKind},
    feedbacks::Feedback,
    observers::ObserversTuple,
    state::State,
    Error,
};
use libafl_bolts::Named;

pub struct AnyTimeoutFeedback;

impl<S> Feedback<S> for AnyTimeoutFeedback
where
    S: State,
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
        match exit_kind {
            ExitKind::Timeout
            | ExitKind::Diff {
                primary: DiffExitKind::Timeout,
                secondary: _,
            }
            | ExitKind::Diff {
                primary: _,
                secondary: DiffExitKind::Timeout,
            } => {
                println!("Timeout");
                Ok(false)
            }
            _ => Ok(true),
        }
    }
}

impl Named for AnyTimeoutFeedback {
    fn name(&self) -> &Cow<'static, str> {
        &Cow::Borrowed("AnyTimeoutFeedback")
    }
}
