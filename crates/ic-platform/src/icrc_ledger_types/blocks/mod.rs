//! Block retrieval and certification response records.
//!
//! Network access, response validation and certification policy remain with
//! consumers.

use super::{ICRC3Value, QueryArchiveFn};
use crate::candid_codec::Nat;
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;

/// A ledger block paired with its unbounded index.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockWithId {
    /// The block's index.
    pub id: Nat,
    /// The block value in the ICRC representation.
    pub block: ICRC3Value,
}

/// Retrieval ranges delegated to an archive callback.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchivedBlocks {
    /// The ranges to request from the archive.
    pub args: Vec<GetBlocksRequest>,
    /// The query callback accepting those ranges.
    pub callback: QueryArchiveFn<Vec<GetBlocksRequest>, GetBlocksResult>,
}

/// A ledger's local blocks and references to archived ranges.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetBlocksResult {
    /// The total log length reported by the ledger.
    pub log_length: Nat,
    /// Blocks returned directly in this response.
    pub blocks: Vec<BlockWithId>,
    /// Ranges to retrieve through archive callbacks.
    pub archived_blocks: Vec<ArchivedBlocks>,
}

/// A requested start index and block count, both unbounded until narrowed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct GetBlocksRequest {
    /// The requested first index.
    pub start: Nat,
    /// The requested number of blocks.
    pub length: Nat,
}

impl GetBlocksRequest {
    /// Narrows each field independently to u64, checking start before length.
    ///
    /// This does not add the fields or validate an interval against a ledger.
    pub fn as_start_and_length(&self) -> Result<(u64, u64), String> {
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

/// Opaque certification material returned by a ledger.
#[derive(Debug, Serialize, Deserialize)]
pub struct ICRC3DataCertificate {
    /// The returned certificate bytes, without local verification.
    pub certificate: ByteBuf,
    /// The returned encoded hash-tree bytes, without local interpretation.
    pub hash_tree: ByteBuf,
}

/// A block-format identifier and its descriptive URL.
#[derive(Debug, Serialize, Deserialize)]
pub struct SupportedBlockType {
    /// The format identifier reported by the ledger.
    pub block_type: String,
    /// The associated URL preserved verbatim.
    pub url: String,
}
