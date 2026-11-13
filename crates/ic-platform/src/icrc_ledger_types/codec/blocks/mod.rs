use super::super::blocks::{
    ArchivedBlocks, BlockWithId, GetBlocksRequest, GetBlocksResult, ICRC3DataCertificate,
    SupportedBlockType,
};
use super::super::{ICRC3Value, QueryArchiveFn};
use crate::candid_codec::Nat;
crate::candid_record!(BlockWithId {23515 => id: Nat, 3036443981 => block: ICRC3Value});
crate::candid_record!(ArchivedBlocks {1081380189 => args: Vec<GetBlocksRequest>, 2131139013 => callback: QueryArchiveFn<Vec<GetBlocksRequest>, GetBlocksResult>});
crate::candid_record!(GetBlocksResult {2799807105 => log_length: Nat, 2817142406 => blocks: Vec<BlockWithId>, 4171053571 => archived_blocks: Vec<ArchivedBlocks>});
crate::candid_record!(GetBlocksRequest {2215343202 => start: Nat, 2668074214 => length: Nat});
crate::candid_record!(ICRC3DataCertificate {457361687 => certificate: serde_bytes::ByteBuf, 2709945007 => hash_tree: serde_bytes::ByteBuf});
crate::candid_record!(SupportedBlockType {5843823 => url: String, 1146110508 => block_type: String});
