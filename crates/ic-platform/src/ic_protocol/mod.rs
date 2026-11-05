//! Native IC HTTP protocol codecs, using upstream transport identities.
//!
//! Every input/output takes a caller-selected byte limit. Decoding does not
//! authenticate signatures, delegations, freshness or request bindings and
//! grants no submission, polling or replay authority. Original bytes remain
//! caller-owned. No generic CBOR value or serializer is exposed.

use crate::certification::cbor::{CborError, decode, encode_framed};
use crate::principal::ExternalPrincipal;

pub use ic_transport_types::{
    Envelope, EnvelopeContent, ReadStateResponse, SubnetMetrics, TransportCallResponse,
};

#[cfg(feature = "ic-protocol-fixtures")]
pub mod fixtures;

/// Decode an IC authentication envelope without verifying or re-encoding it.
pub fn decode_envelope(bytes: &[u8], max_bytes: usize) -> Result<Envelope<'static>, CborError> {
    decode(bytes, max_bytes)
}

/// Inspect a call's known fields with strict unknown-field admission. Outer
/// authentication metadata is not checked. Consumers retain sender-info policy,
/// expected content/request-ID binding and custody of the original signed bytes.
pub fn decode_call_content_strict(
    bytes: &[u8],
    max_bytes: usize,
) -> Result<EnvelopeContent, CborError> {
    use serde::Deserialize;
    use serde::de::Error as _;
    #[derive(Deserialize)]
    struct EnvelopeWire {
        content: CallWire,
    }
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct CallWire {
        request_type: String,
        nonce: Option<serde_bytes::ByteBuf>,
        ingress_expiry: u64,
        sender: serde_bytes::ByteBuf,
        canister_id: serde_bytes::ByteBuf,
        method_name: String,
        arg: serde_bytes::ByteBuf,
        sender_info: Option<ic_transport_types::SenderInfo>,
    }
    let value: EnvelopeWire = decode(bytes, max_bytes)?;
    let call = value.content;
    let invalid = || CborError::Decode(serde_cbor::Error::custom("invalid IC call content"));
    if call.request_type != "call" {
        return Err(invalid());
    }
    Ok(EnvelopeContent::Call {
        nonce: call.nonce.map(serde_bytes::ByteBuf::into_vec),
        ingress_expiry: call.ingress_expiry,
        sender: ExternalPrincipal::try_from_slice(&call.sender).map_err(|_| invalid())?,
        canister_id: ExternalPrincipal::try_from_slice(&call.canister_id).map_err(|_| invalid())?,
        method_name: call.method_name,
        arg: call.arg.into_vec(),
        sender_info: call.sender_info,
    })
}

/// Encode an IC envelope with the same self-described framing as the SDK.
pub fn encode_envelope(envelope: &Envelope<'_>, max_bytes: usize) -> Result<Vec<u8>, CborError> {
    encode_framed(envelope, max_bytes, true)
}

/// Decode an HTTP call response; acceptance is not permission to poll or retry.
pub fn decode_call_response(
    bytes: &[u8],
    max_bytes: usize,
) -> Result<TransportCallResponse, CborError> {
    decode(bytes, max_bytes)
}

/// Decode a read-state response, retaining its original certificate bytes.
pub fn decode_read_state_response(
    bytes: &[u8],
    max_bytes: usize,
) -> Result<ReadStateResponse, CborError> {
    decode(bytes, max_bytes)
}

/// Decode the IC subnet-metrics leaf, preserving the upstream u128 cycle count.
pub fn decode_subnet_metrics(bytes: &[u8], max_bytes: usize) -> Result<SubnetMetrics, CborError> {
    decode(bytes, max_bytes)
}

/// Decode an IC certified controllers leaf; authentication and controller
/// counts, ordering, uniqueness and authority remain consumer-owned.
pub fn decode_controllers(
    bytes: &[u8],
    max_bytes: usize,
) -> Result<Vec<ExternalPrincipal>, CborError> {
    decode(bytes, max_bytes)
}

#[cfg(test)]
mod tests;
