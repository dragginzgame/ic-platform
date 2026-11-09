//! Selected upstream ICRC implementation; see ../NOTICE.md.
#![expect(
    missing_docs,
    reason = "Selected upstream ICRC declarations contain undocumented public fields and variants."
)]

use crate::candid_codec::Nat;
use crate::principal::Principal;
use serde::Deserialize;
use serde::Serialize;
use std::marker::PhantomData;

/// Details on the callback function using which archived blocks can be retrieved. Returned as part
/// of [`super::GetBlocksResult`] from the
/// [`icrc3_get_blocks`](https://github.com/dfinity/ICRC-1/blob/main/standards/ICRC-3/README.md)
/// endpoint.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "crate::candid_codec::references::Func")]
pub struct QueryArchiveFn<Input, Output> {
    pub canister_id: Principal,
    pub method: String,
    pub _marker: PhantomData<(Input, Output)>,
}

impl<Input, Output> PartialOrd for QueryArchiveFn<Input, Output>
where
    Input: Eq,
    Output: Eq,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<Input, Output> Ord for QueryArchiveFn<Input, Output>
where
    Input: Eq,
    Output: Eq,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.canister_id.cmp(&other.canister_id) {
            std::cmp::Ordering::Equal => self.method.cmp(&other.method),
            c => c,
        }
        // the _marker doesn't matter
    }
}

impl<Input, Output> QueryArchiveFn<Input, Output> {
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
        Self {
            canister_id: self.canister_id,
            method: self.method.clone(),
            _marker: PhantomData,
        }
    }
}

impl<Input, Output> From<QueryArchiveFn<Input, Output>> for crate::candid_codec::references::Func {
    fn from(archive_fn: QueryArchiveFn<Input, Output>) -> Self {
        let p: &Principal = &Principal::try_from(archive_fn.canister_id.as_ref())
            .expect("could not deserialize principal");
        Self {
            principal: *p,
            method: archive_fn.method,
        }
    }
}

impl<Input, Output> TryFrom<crate::candid_codec::references::Func>
    for QueryArchiveFn<Input, Output>
{
    type Error = String;
    fn try_from(func: crate::candid_codec::references::Func) -> Result<Self, Self::Error> {
        let canister_id = Principal::try_from(func.principal.as_slice())
            .map_err(|e| format!("principal is not a canister id: {e}"))?;
        Ok(QueryArchiveFn {
            canister_id,
            method: func.method,
            _marker: PhantomData,
        })
    }
}

/// The argument for the
/// [`icrc3_get_archives`](https://github.com/dfinity/ICRC-1/blob/main/standards/ICRC-3/README.md)
/// endpoint.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct GetArchivesArgs {
    // The last archive seen by the client.
    // The Ledger will return archives coming
    // after this one if set, otherwise it
    // will return the first archives.
    pub from: Option<Principal>,
}

/// The information returned as part of the return value for the
/// [`icrc3_get_archives`](https://github.com/dfinity/ICRC-1/blob/main/standards/ICRC-3/README.md)
/// endpoint.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ICRC3ArchiveInfo {
    // The id of the archive
    pub canister_id: Principal,

    // The first block in the archive
    pub start: Nat,

    // The last block in the archive
    pub end: Nat,
}

impl<Input: crate::candid_codec::Encode, Output: crate::candid_codec::Encode>
    crate::candid_codec::Encode for QueryArchiveFn<Input, Output>
{
    const TYPE: &'static crate::candid_codec::Type = &crate::candid_codec::Type::Function {
        args: &[Input::TYPE],
        results: &[Output::TYPE],
        annotations: &[1],
    };
    fn write(
        &self,
        writer: &mut crate::candid_codec::Writer,
    ) -> Result<(), crate::candid_codec::Error> {
        crate::candid_codec::references::Func::write_parts(&self.canister_id, &self.method, writer)
    }
}
impl<'a, Input: crate::candid_codec::Encode, Output: crate::candid_codec::Encode>
    crate::candid_codec::decoder::Decode<'a> for QueryArchiveFn<Input, Output>
{
    fn read(
        reader: &mut crate::candid_codec::decoder::Reader<'a>,
        wire: i64,
    ) -> Result<Self, crate::candid_codec::decoder::Error> {
        let value = crate::candid_codec::references::Func::read(
            reader,
            wire,
            <Self as crate::candid_codec::Encode>::TYPE,
        )?;
        Ok(Self::new(value.principal, value.method))
    }
}

#[cfg(test)]
impl<Input, Output> From<QueryArchiveFn<Input, Output>> for candid::types::reference::Func {
    fn from(value: QueryArchiveFn<Input, Output>) -> Self {
        Self {
            principal: candid::Principal::from_slice(value.canister_id.as_slice()),
            method: value.method,
        }
    }
}
#[cfg(test)]
impl<Input, Output> TryFrom<candid::types::reference::Func> for QueryArchiveFn<Input, Output> {
    type Error = String;
    fn try_from(value: candid::types::reference::Func) -> Result<Self, Self::Error> {
        Self::try_from(crate::candid_codec::references::Func::from(value))
    }
}
