use core::fmt;
use std::{
    borrow::Cow,
    ffi::{OsStr, OsString},
    fmt::{Display, Formatter},
    hash::{DefaultHasher, Hash, Hasher},
};

use serde::{Deserialize, Serialize};

use libafl::{
    generators::Generator,
    inputs::{HasMutatorBytes, Input},
    mutators::{havoc_mutations, MutationResult, Mutator},
    state::HasRand,
    Error, SerdeAny,
};

use libafl_bolts::{
    prelude::Rand,
    tuples::{tuple_list_type, Append},
    HasLen, Named,
};

use crate::generic::{executor::ExtractsToCommand, stdio::vec_string_mapper};

/// An [`Input`] implementation for coreutils' `base64`
#[derive(Serialize, Deserialize, Clone, Debug, Hash, SerdeAny)]
pub struct Base64Input {
    pub input: Vec<u8>,
    pub decode: bool,
    pub ignore_garbage: bool,
    pub wrap: Option<i8>,
}

impl Display for Base64Input {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            f,
            "input: '{}'",
            vec_string_mapper(&Some(self.input.clone()))
        )?;
        if self.decode {
            write!(f, ", decode")?;
        }
        if self.ignore_garbage {
            write!(f, ", ignore_garbage")?;
        }
        if let Some(wrap) = self.wrap {
            write!(f, ", wrap: {}", wrap)?;
        }
        Ok(())
    }
}

impl Input for Base64Input {
    #[must_use]
    fn generate_name(&self, _idx: usize) -> String {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }
}

impl ExtractsToCommand for Base64Input {
    #[must_use]
    fn get_stdin(&self) -> &Vec<u8> {
        &self.input
    }

    #[must_use]
    fn get_args<'a>(&self) -> Vec<Cow<'a, OsStr>> {
        let mut args = Vec::with_capacity(4);
        if self.decode {
            args.push(Cow::Borrowed(OsStr::new("-d")))
        }
        if self.ignore_garbage {
            args.push(Cow::Borrowed(OsStr::new("-i")))
        }
        if let Some(w) = &self.wrap {
            args.push(Cow::Borrowed(OsStr::new("-w")));
            args.push(Cow::Owned(OsString::from(w.to_string())))
        }
        args
    }
}

impl HasMutatorBytes for Base64Input {
    fn bytes(&self) -> &[u8] {
        &self.input
    }

    fn bytes_mut(&mut self) -> &mut [u8] {
        &mut self.input
    }

    fn resize(&mut self, new_len: usize, value: u8) {
        self.input.resize(new_len, value)
    }

    fn extend<'a, I: IntoIterator<Item = &'a u8>>(&mut self, iter: I) {
        self.input.extend(iter)
    }

    fn splice<R, I>(
        &mut self,
        range: R,
        replace_with: I,
    ) -> libafl::prelude::alloc::vec::Splice<'_, I::IntoIter>
    where
        R: std::ops::RangeBounds<usize>,
        I: IntoIterator<Item = u8>,
    {
        self.input.splice(range, replace_with)
    }

    fn drain<R>(&mut self, range: R) -> libafl::prelude::alloc::vec::Drain<'_, u8>
    where
        R: std::ops::RangeBounds<usize>,
    {
        self.input.drain(range)
    }
}

impl HasLen for Base64Input {
    fn len(&self) -> usize {
        self.input.len()
    }
}

impl Base64Input {
    #[must_use]
    pub fn new(input: &[u8], decode: bool, ignore_garbage: bool, wrap: Option<i8>) -> Self {
        Self {
            input: Vec::from(input),
            decode,
            ignore_garbage,
            wrap,
        }
    }
}

pub struct Base64Generator {
    input_size: u32,
}

impl Base64Generator {
    pub fn new(input_size: u32) -> Self {
        Self { input_size }
    }
}

impl<S> Generator<Base64Input, S> for Base64Generator
where
    S: HasRand,
{
    fn generate(&mut self, state: &mut S) -> Result<Base64Input, Error> {
        let input = &generate_bytes(state, self.input_size);

        let rand = state.rand_mut();
        let decode = rand.coinflip(0.5);
        let ignore_garbage = rand.coinflip(0.5);
        let wrap = rand
            .coinflip(0.5)
            .then(|| rand.between(i8::MIN as usize, i8::MAX as usize) as i8);
        Ok(Base64Input::new(input, decode, ignore_garbage, wrap))
    }
}

pub struct Base64FlipDecodeMutator;
impl<S> Mutator<Base64Input, S> for Base64FlipDecodeMutator
where
    S: HasRand,
{
    fn mutate(&mut self, _state: &mut S, input: &mut Base64Input) -> Result<MutationResult, Error> {
        input.decode = !input.decode;
        Ok(MutationResult::Mutated)
    }
}

impl Named for Base64FlipDecodeMutator {
    fn name(&self) -> &Cow<'static, str> {
        &Cow::Borrowed("Base64FlipDecodeMutator")
    }
}
pub struct Base64FlipIgnoreGarbageMutator;
impl<S> Mutator<Base64Input, S> for Base64FlipIgnoreGarbageMutator
where
    S: HasRand,
{
    fn mutate(&mut self, _state: &mut S, input: &mut Base64Input) -> Result<MutationResult, Error> {
        input.ignore_garbage = !input.ignore_garbage;
        Ok(MutationResult::Mutated)
    }
}

impl Named for Base64FlipIgnoreGarbageMutator {
    fn name(&self) -> &Cow<'static, str> {
        &Cow::Borrowed("Base64FlipIgnoreGarbageMutator")
    }
}

pub struct Base64FlipWrapMutator;
impl<S> Mutator<Base64Input, S> for Base64FlipWrapMutator
where
    S: HasRand,
{
    fn mutate(&mut self, state: &mut S, input: &mut Base64Input) -> Result<MutationResult, Error> {
        match &input.wrap {
            None => {
                input.wrap =
                    Some(state.rand_mut().between(i8::MIN as usize, i8::MAX as usize) as i8);
                Ok(MutationResult::Mutated)
            }
            Some(_e) => {
                input.wrap = None;
                Ok(MutationResult::Mutated)
            }
        }
    }
}

impl Named for Base64FlipWrapMutator {
    fn name(&self) -> &Cow<'static, str> {
        &Cow::Borrowed("Base64FlipWrapMutator")
    }
}

pub struct Base64WrapContentMutator;

impl<S> Mutator<Base64Input, S> for Base64WrapContentMutator
where
    S: HasRand,
{
    fn mutate(&mut self, state: &mut S, input: &mut Base64Input) -> Result<MutationResult, Error> {
        match input.wrap {
            Some(_) => {
                input.wrap =
                    Some(state.rand_mut().between(i8::MIN as usize, i8::MAX as usize) as i8);
                Ok(MutationResult::Mutated)
            }
            None => Ok(MutationResult::Skipped),
        }
    }
}

impl Named for Base64WrapContentMutator {
    fn name(&self) -> &Cow<'static, str> {
        &Cow::Borrowed("Base64WrapContentMutator")
    }
}

fn generate_bytes<S: HasRand>(state: &mut S, len: u32) -> Vec<u8> {
    (0..len)
        .map(|_e| state.rand_mut().below(u8::MAX as usize + 1) as u8)
        .collect::<Vec<_>>()
}

pub fn base64_mutators() -> tuple_list_type!(
    libafl::mutators::BitFlipMutator,
    libafl::mutators::ByteFlipMutator,
    libafl::mutators::ByteIncMutator,
    libafl::mutators::ByteDecMutator,
    libafl::mutators::ByteNegMutator,
    libafl::mutators::ByteRandMutator,
    libafl::mutators::ByteAddMutator,
    libafl::mutators::WordAddMutator,
    libafl::mutators::DwordAddMutator,
    libafl::mutators::QwordAddMutator,
    libafl::mutators::ByteInterestingMutator,
    libafl::mutators::WordInterestingMutator,
    libafl::mutators::DwordInterestingMutator,
    libafl::mutators::BytesDeleteMutator,
    libafl::mutators::BytesDeleteMutator,
    libafl::mutators::BytesDeleteMutator,
    libafl::mutators::BytesDeleteMutator,
    libafl::mutators::BytesExpandMutator,
    libafl::mutators::BytesInsertMutator,
    libafl::mutators::BytesRandInsertMutator,
    libafl::mutators::BytesSetMutator,
    libafl::mutators::BytesRandSetMutator,
    libafl::mutators::BytesCopyMutator,
    libafl::mutators::BytesInsertCopyMutator,
    libafl::mutators::BytesSwapMutator,
    libafl::mutators::CrossoverInsertMutator<Base64Input>,
    libafl::mutators::CrossoverReplaceMutator<Base64Input>,
    Base64FlipDecodeMutator,
    Base64FlipIgnoreGarbageMutator,
    Base64FlipWrapMutator,
    Base64WrapContentMutator
) {
    havoc_mutations()
        .append(Base64FlipDecodeMutator)
        .append(Base64FlipIgnoreGarbageMutator)
        .append(Base64FlipWrapMutator)
        .append(Base64WrapContentMutator)
}
