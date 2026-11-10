//! Typed archive references and archive-list records.
//!
//! References preserve caller-supplied identities and methods without imposing
//! network, authority, budget or retry policy.

use crate::{
    candid_codec::{
        Encode, Error, Nat, Type, Writer,
        decoder::{Decode, Error as DecodeError, Reader},
        references::Func,
    },
    principal::Principal,
};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, marker::PhantomData};

/// A query callback taking one Input and returning one Output.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "crate::candid_codec::references::Func")]
pub struct QueryArchiveFn<Input, Output> {
    /// The canonical Principal to which the callback refers.
    pub canister_id: Principal,
    /// The method name, preserved verbatim.
    pub method: String,
    /// Type-level input and output information; serialized as a unit marker.
    pub _marker: PhantomData<(Input, Output)>,
}

impl<Input, Output> QueryArchiveFn<Input, Output> {
    /// Constructs a callback without imposing canister or method restrictions.
    pub fn new(canister_id: Principal, method: impl Into<String>) -> Self {
        Self {
            canister_id,
            method: method.into(),
            _marker: PhantomData,
        }
    }
}

impl<Input, Output> Clone for QueryArchiveFn<Input, Output> {
    fn clone(&self) -> Self {
        Self::new(self.canister_id, self.method.clone())
    }
}

impl<Input: Eq, Output: Eq> Ord for QueryArchiveFn<Input, Output> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.canister_id
            .cmp(&other.canister_id)
            .then_with(|| self.method.cmp(&other.method))
    }
}

impl<Input: Eq, Output: Eq> PartialOrd for QueryArchiveFn<Input, Output> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<Input, Output> From<QueryArchiveFn<Input, Output>> for Func {
    fn from(reference: QueryArchiveFn<Input, Output>) -> Self {
        Self {
            principal: reference.canister_id,
            method: reference.method,
        }
    }
}

impl<Input, Output> TryFrom<Func> for QueryArchiveFn<Input, Output> {
    type Error = String;

    fn try_from(reference: Func) -> Result<Self, Self::Error> {
        Ok(Self::new(reference.principal, reference.method))
    }
}

impl<Input: Encode, Output: Encode> Encode for QueryArchiveFn<Input, Output> {
    const TYPE: &'static Type = &Type::Function {
        args: &[Input::TYPE],
        results: &[Output::TYPE],
        annotations: &[1],
    };

    fn write(&self, writer: &mut Writer) -> Result<(), Error> {
        Func::write_parts(&self.canister_id, &self.method, writer)
    }
}

impl<'a, Input: Encode, Output: Encode> Decode<'a> for QueryArchiveFn<Input, Output> {
    fn read(reader: &mut Reader<'a>, wire: i64) -> Result<Self, DecodeError> {
        let reference = Func::read(reader, wire, Self::TYPE)?;
        Ok(Self::new(reference.principal, reference.method))
    }
}

#[cfg(test)]
impl<Input, Output> From<QueryArchiveFn<Input, Output>> for candid::types::reference::Func {
    fn from(reference: QueryArchiveFn<Input, Output>) -> Self {
        Self {
            principal: candid::Principal::from_slice(reference.canister_id.as_slice()),
            method: reference.method,
        }
    }
}

#[cfg(test)]
impl<Input, Output> TryFrom<candid::types::reference::Func> for QueryArchiveFn<Input, Output> {
    type Error = String;

    fn try_from(reference: candid::types::reference::Func) -> Result<Self, Self::Error> {
        Self::try_from(Func::from(reference))
    }
}

/// A request to list archive references.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetArchivesArgs {
    /// The optional Principal identifying the caller's starting point.
    pub from: Option<Principal>,
}

/// An archive's reported canister identity and natural-number range.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ICRC3ArchiveInfo {
    /// The Principal of the archive canister.
    pub canister_id: Principal,
    /// The reported start of the archive's range.
    pub start: Nat,
    /// The reported end of the archive's range.
    pub end: Nat,
}

#[cfg(test)]
mod tests;
