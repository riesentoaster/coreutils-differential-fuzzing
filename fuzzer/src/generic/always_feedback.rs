use std::borrow::Cow;

use libafl::{
    corpus::Testcase,
    events::EventFirer,
    executors::ExitKind,
    feedbacks::Feedback,
    inputs::{HasBytesVec, UsesInput},
    observers::ObserversTuple,
    state::State,
    Error,
};
use libafl_bolts::Named;

pub struct AlwaysFeedback;
impl<S> Feedback<S> for AlwaysFeedback
where
    S: State + UsesInput,
    S::Input: HasBytesVec,
{
    fn is_interesting<EM, OT>(
        &mut self,
        _state: &mut S,
        _manager: &mut EM,
        _input: &S::Input,
        _observers: &OT,
        _exit_kind: &ExitKind,
    ) -> Result<bool, Error>
    where
        EM: EventFirer<State = S>,
        OT: ObserversTuple<S>,
    {
        Ok(false)
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
        println!(
            "New corpus entry with len {}",
            testcase.input().as_ref().unwrap().bytes().len()
        );
        Ok(())
    }
}

impl Named for AlwaysFeedback {
    fn name(&self) -> &Cow<'static, str> {
        &Cow::Borrowed("AlwaysFeedback")
    }
}
