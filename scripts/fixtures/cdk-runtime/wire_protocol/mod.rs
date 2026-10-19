//! Consumer-owned fixture identities and allowances, shared by both RPC ends.

pub const ECHO: u64 = 1;
pub const SET: u64 = 2;
pub const OBSERVE: u64 = 3;
pub const AWAIT: u64 = 4;
pub const ENCODE_TRAP: u64 = 5;
pub const TIGHT: u64 = 6;
pub const ALLOCATION: u64 = 7;
pub const DEPTH: u64 = 8;
pub const REQUEST: ic_wire::Limits = ic_wire::Limits {
    bytes: 1024,
    allocation: 16 * 1024,
    nodes: 128,
    depth: 8,
};
pub const RESPONSE: ic_wire::Limits = REQUEST;
pub fn request_limits(id: u64) -> ic_wire::Limits {
    match id {
        TIGHT => ic_wire::Limits {
            nodes: 2,
            ..REQUEST
        },
        ALLOCATION => ic_wire::Limits {
            allocation: REQUEST.bytes,
            ..REQUEST
        },
        DEPTH => ic_wire::Limits {
            depth: 0,
            ..REQUEST
        },
        _ => REQUEST,
    }
}
