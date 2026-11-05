//! Deliberate native IC protocol fixtures, never authentication assertions.
//! Select only in development dependencies. These APIs cannot encode application DTOs.

use super::*;
use crate::certification::{Certificate, HashTree};
use ic_transport_types::{QueryResponse, RejectResponse, ReplyResponse};
use serde::Serialize;

/// Encode an unsigned replied query with exact opaque reply bytes.
pub fn query_reply(arg: &[u8], max_bytes: usize) -> Result<Vec<u8>, CborError> {
    encode_framed(
        &QueryResponse::Replied {
            reply: ReplyResponse { arg: arg.to_vec() },
            signatures: Vec::new(),
        },
        max_bytes,
        false,
    )
}

/// Encode an unsigned query rejection with its original structured error fields.
pub fn query_rejection(reject: &RejectResponse, max_bytes: usize) -> Result<Vec<u8>, CborError> {
    encode_framed(
        &QueryResponse::Rejected {
            reject: reject.clone(),
            signatures: Vec::new(),
        },
        max_bytes,
        false,
    )
}

/// Encode a read-state response containing caller-selected certificate bytes.
pub fn read_state_response(certificate: &[u8], max_bytes: usize) -> Result<Vec<u8>, CborError> {
    encode_framed(
        &ReadStateResponse {
            certificate: certificate.to_vec(),
        },
        max_bytes,
        false,
    )
}

/// Encode the IC certified controllers leaf, including deliberate duplicates.
pub fn controllers(value: &[ExternalPrincipal], max_bytes: usize) -> Result<Vec<u8>, CborError> {
    encode_framed(&value, max_bytes, false)
}

/// Encode subnet metrics using the unchanged upstream u128 representation.
pub fn subnet_metrics(value: &SubnetMetrics, max_bytes: usize) -> Result<Vec<u8>, CborError> {
    encode_framed(value, max_bytes, false)
}

/// Encode the IC canister-ranges leaf, without asserting any range policy.
pub fn canister_ranges(
    ranges: &[(crate::Principal, crate::Principal)],
    max_bytes: usize,
) -> Result<Vec<u8>, CborError> {
    encode_framed(&ranges, max_bytes, false)
}

/// Certificate with one unknown field, for strict admission regressions.
pub fn certificate_with_unknown_field(
    value: &Certificate,
    max_bytes: usize,
) -> Result<Vec<u8>, CborError> {
    #[derive(Serialize)]
    struct Unknown<'a> {
        #[serde(flatten)]
        certificate: &'a Certificate,
        extra: (),
    }
    encode_framed(
        &Unknown {
            certificate: value,
            extra: (),
        },
        max_bytes,
        false,
    )
}

/// Indefinite certificate map with a repeated tree, for duplicate-field refusal.
pub fn certificate_with_duplicate_tree(
    value: &Certificate,
    max_bytes: usize,
) -> Result<Vec<u8>, CborError> {
    use serde::ser::{SerializeMap, Serializer};
    struct Duplicate<'a>(&'a Certificate);
    impl Serialize for Duplicate<'_> {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let mut map = serializer.serialize_map(None)?;
            map.serialize_entry("tree", &self.0.tree)?;
            map.serialize_entry("signature", &serde_bytes::Bytes::new(&self.0.signature))?;
            if let Some(delegation) = &self.0.delegation {
                map.serialize_entry("delegation", delegation)?;
            }
            map.serialize_entry("tree", &self.0.tree)?;
            map.end()
        }
    }
    encode_framed(&Duplicate(value), max_bytes, false)
}

/// Sequence-form canister signature admitted by the upstream decoder.
pub fn sequence_canister_signature(
    certificate: &[u8],
    tree: &HashTree,
    max_bytes: usize,
) -> Result<Vec<u8>, CborError> {
    encode_framed(
        &(serde_bytes::Bytes::new(certificate), tree),
        max_bytes,
        true,
    )
}
