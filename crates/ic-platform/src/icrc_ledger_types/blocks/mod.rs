//! Selected upstream ICRC implementation; see ../NOTICE.md.
#![allow(
    missing_docs,
    reason = "Retained upstream declarations and documentation"
)]

use super::{ICRC3Value, QueryArchiveFn};
use candid::{CandidType, Deserialize, Nat};
use serde::Serialize;

/// A block with an ID. Returned as part of [`GetBlocksResult`].
#[derive(Debug, CandidType, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlockWithId {
    pub id: Nat,
    pub block: ICRC3Value,
}

/// Information about where to find archived blocks. Returned as part of [`GetBlocksResult`].
#[derive(Debug, CandidType, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArchivedBlocks {
    pub args: Vec<GetBlocksRequest>,
    pub callback: QueryArchiveFn<Vec<GetBlocksRequest>, GetBlocksResult>,
}

/// The result type for the
/// [ICRC-3 `icrc3_get_blocks`](https://github.com/dfinity/ICRC-1/blob/main/standards/ICRC-3/README.md#icrc3_get_blocks)
/// endpoint.
#[derive(Debug, CandidType, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GetBlocksResult {
    pub log_length: Nat,
    pub blocks: Vec<BlockWithId>,
    pub archived_blocks: Vec<ArchivedBlocks>,
}

/// The arguments for the
/// [ICRC-3 `icrc3_get_blocks`](https://github.com/dfinity/ICRC-1/blob/main/standards/ICRC-3/README.md#icrc3_get_blocks)
/// endpoint.
#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct GetBlocksRequest {
    pub start: Nat,
    pub length: Nat,
}

impl GetBlocksRequest {
    pub fn as_start_and_length(&self) -> Result<(u64, u64), String> {
        use num_traits::cast::ToPrimitive;

        let start = self.start.0.to_u64().ok_or_else(|| {
            format!(
                "transaction index {} is too large, max allowed: {}",
                self.start,
                u64::MAX
            )
        })?;
        let length = self.length.0.to_u64().ok_or_else(|| {
            format!(
                "requested length {} is too large, max allowed: {}",
                self.length,
                u64::MAX
            )
        })?;
        Ok((start, length))
    }
}

/// The data certificate returned from the
/// [ICRC-3 `icrc3_get_tip_certificate`](https://github.com/dfinity/ICRC-1/blob/main/standards/ICRC-3/README.md#icrc3_get_tip_certificate)
/// endpoint.
#[derive(Debug, CandidType, Serialize, Deserialize)]
pub struct ICRC3DataCertificate {
    pub certificate: serde_bytes::ByteBuf,
    pub hash_tree: serde_bytes::ByteBuf,
}

/// The return type of the
/// [ICRC-3 `icrc3_supported_block_types`](https://github.com/dfinity/ICRC-1/blob/main/standards/ICRC-3/README.md#icrc3_supported_block_types)
/// endpoint.
#[derive(Debug, CandidType, Serialize, Deserialize)]
pub struct SupportedBlockType {
    pub block_type: String,
    pub url: String,
}
