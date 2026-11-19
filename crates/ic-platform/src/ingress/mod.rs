//! Strict native ingress packet admission. Decoding does not authenticate a
//! sender, delegation or certificate. Original bytes remain available for replay.
//! The inclusive input ceiling is checked before parsing; it is not a process
//! heap or work quota. Serde CBOR's recursion ceiling remains enabled.

#[cfg(test)]
mod tests;
mod wire;

use candidate_types::Principal;
use serde::{Deserialize, Serialize};

/// Offline envelope construction failure, without a partial output.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("ingress encoding refusal: {self:?}")]
pub enum EncodeError {
    /// Complete encoded output exceeds the inclusive byte ceiling.
    ByteLimit,
    /// Storage for the admitted output could not be reserved.
    Allocation,
    /// Serialization refused the supplied fields.
    Packet,
}

/// Construct one unverified envelope from caller-supplied fields and signatures.
/// Counts the complete CBOR output before reserving its buffer. No signature,
/// identity, path, expiry or delegation policy is imposed; even invalid signature
/// fixtures can be encoded. The caller retains the returned bytes before dispatch.
/// This constructs new bytes and must not replace an already retained envelope.
///
/// # Errors
/// Refuses output exceeding `maximum_bytes`, allocation or serialization failure.
pub fn encode_envelope(envelope: &Envelope, maximum_bytes: usize) -> Result<Vec<u8>, EncodeError> {
    encode(envelope, maximum_bytes)
}

// Internal reuse for the native client's selected query and envelope records.
pub(crate) fn encode<T: Serialize>(value: &T, maximum: usize) -> Result<Vec<u8>, EncodeError> {
    struct Count {
        count: usize,
        maximum: usize,
        exceeded: bool,
    }
    impl std::io::Write for Count {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            let Some(next) = self
                .count
                .checked_add(bytes.len())
                .filter(|n| *n <= self.maximum)
            else {
                self.exceeded = true;
                return Err(std::io::Error::other("message limit"));
            };
            self.count = next;
            Ok(bytes.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut count = Count {
        count: 0,
        maximum,
        exceeded: false,
    };
    if serde_cbor::to_writer(&mut count, value).is_err() {
        return Err(if count.exceeded {
            EncodeError::ByteLimit
        } else {
            EncodeError::Packet
        });
    }
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(count.count)
        .map_err(|_| EncodeError::Allocation)?;
    bytes.resize(count.count, 0);
    let mut output = bytes.as_mut_slice();
    serde_cbor::to_writer(&mut output, value).map_err(|_| EncodeError::Packet)?;
    if !output.is_empty() {
        return Err(EncodeError::Packet);
    }
    Ok(bytes)
}

/// Admission failure without a partially decoded packet.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("ingress packet refusal: {self:?}")]
pub enum Error {
    /// Complete input exceeds the caller's ceiling.
    ByteLimit,
    /// Malformed, duplicate, unknown, wrong-type, truncated or trailing data.
    Packet,
    /// Content has a different request kind than the requested operation.
    Kind,
    /// Upstream request-ID serialization refused the content.
    RequestId,
}

/// Decoded data alongside the exact input, without normalization or re-encoding.
#[derive(Debug)]
pub struct Packet<'a, T> {
    /// Original complete input including any self-describing CBOR tag.
    pub original: &'a [u8],
    /// Unauthenticated decoded fields.
    pub value: T,
}

/// Supported original ingress content; queries are deliberately outside this API.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "request_type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Content {
    /// Replicated call content. Method authorization and replay policy are external.
    Call {
        /// Exact optional nonce, distinguishing absent from empty.
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            serialize_with = "serde_bytes::serialize",
            deserialize_with = "wire::optional_bytes"
        )]
        nonce: Option<Vec<u8>>,
        /// Caller-supplied expiry in nanoseconds; no freshness claim.
        ingress_expiry: u64,
        /// Canonical sender identity.
        #[serde(deserialize_with = "wire::principal")]
        sender: Principal,
        /// Canonical destination identity.
        #[serde(deserialize_with = "wire::principal")]
        canister_id: Principal,
        /// Exact method text.
        method_name: String,
        /// Exact argument bytes.
        #[serde(
            serialize_with = "serde_bytes::serialize",
            deserialize_with = "wire::bytes"
        )]
        arg: Vec<u8>,
        /// Presence remains visible for caller-owned refusal or verification.
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            deserialize_with = "wire::optional_map"
        )]
        sender_info: Option<SenderInfo>,
    },
    /// Original status lookup content. Path authorization remains external.
    ReadState {
        /// Caller-supplied expiry in nanoseconds.
        ingress_expiry: u64,
        /// Canonical sender identity.
        #[serde(deserialize_with = "wire::principal")]
        sender: Principal,
        /// Exact nested path labels; only CBOR byte strings are admitted.
        #[serde(
            serialize_with = "wire::serialize_paths",
            deserialize_with = "wire::paths"
        )]
        paths: Vec<Vec<Vec<u8>>>,
    },
}
impl Content {
    /// Derive representation-independent identity using the original upstream
    /// implementation. No hash implementation or signing policy is supplied here.
    ///
    /// # Errors
    /// Returns `RequestId` if upstream serialization refuses these fields.
    pub fn request_id(&self) -> Result<[u8; 32], Error> {
        ic_transport_types::to_request_id(self)
            .map(|id| *id)
            .map_err(|_| Error::RequestId)
    }

    /// Return the upstream request ID's domain-separated signing bytes.
    ///
    /// # Errors
    /// Returns `RequestId` if upstream serialization refuses these fields.
    pub fn signable(&self) -> Result<Vec<u8>, Error> {
        ic_transport_types::to_request_id(self)
            .map(|id| id.signable())
            .map_err(|_| Error::RequestId)
    }
}

/// Canister-certified sender information, still wholly unverified.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SenderInfo {
    /// Exact supplied information.
    #[serde(
        serialize_with = "serde_bytes::serialize",
        deserialize_with = "wire::bytes"
    )]
    pub info: Vec<u8>,
    /// Exact signer bytes; caller owns their interpretation.
    #[serde(
        serialize_with = "serde_bytes::serialize",
        deserialize_with = "wire::bytes"
    )]
    pub signer: Vec<u8>,
    /// Original signature bytes.
    #[serde(
        serialize_with = "serde_bytes::serialize",
        deserialize_with = "wire::bytes"
    )]
    pub sig: Vec<u8>,
}

/// Unverified delegation permissions.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Permissions {
    /// Upstream queries-only marker.
    Queries,
    /// Upstream all-request marker.
    All,
}
/// Unverified delegation fields; no chain or target authorization is inferred.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Delegation {
    /// Delegated key bytes.
    #[serde(
        serialize_with = "serde_bytes::serialize",
        deserialize_with = "wire::bytes"
    )]
    pub pubkey: Vec<u8>,
    /// Raw expiration.
    pub expiration: u64,
    /// Absent and present-empty target lists remain distinct.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "wire::targets"
    )]
    pub targets: Option<Vec<Principal>>,
    /// Optional upstream permission marker.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "wire::present"
    )]
    pub permissions: Option<Permissions>,
}
/// Original signed delegation, without a signature verification claim.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignedDelegation {
    /// Delegation fields in original order in the envelope chain.
    #[serde(deserialize_with = "wire::map")]
    pub delegation: Delegation,
    /// Exact signature bytes.
    #[serde(
        serialize_with = "serde_bytes::serialize",
        deserialize_with = "wire::bytes"
    )]
    pub signature: Vec<u8>,
}
/// Original authentication envelope; presence is preserved even for empty fields.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Envelope {
    /// Unauthenticated request content.
    #[serde(deserialize_with = "wire::map")]
    pub content: Content,
    /// Optional original key bytes.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        serialize_with = "serde_bytes::serialize",
        deserialize_with = "wire::optional_bytes"
    )]
    pub sender_pubkey: Option<Vec<u8>>,
    /// Optional original signature bytes.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        serialize_with = "serde_bytes::serialize",
        deserialize_with = "wire::optional_bytes"
    )]
    pub sender_sig: Option<Vec<u8>>,
    /// Optional original ordered delegation chain.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "wire::delegations"
    )]
    pub sender_delegation: Option<Vec<SignedDelegation>>,
}

/// Synchronous ingress outcome; certificate bytes have not been authenticated.
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum CallResponse {
    /// Original certified response envelope.
    Replied {
        /// Opaque original certificate.
        #[serde(deserialize_with = "wire::bytes")]
        certificate: Vec<u8>,
    },
    /// Forwarded ingress with no completion claim. This does not authorize resubmission.
    Accepted,
    /// Uncertified rejection, distinct from a certified executed rejection.
    NonReplicatedRejection {
        /// Raw code retained for caller-selected policy.
        reject_code: u64,
        /// Exact diagnostic text.
        reject_message: String,
        /// Optional diagnostic code, including empty text if present.
        #[serde(default, deserialize_with = "wire::present")]
        error_code: Option<String>,
    },
}
/// Unauthenticated read-state response.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadStateResponse {
    /// Opaque original certificate; authenticate separately with explicit policy.
    #[serde(deserialize_with = "wire::bytes")]
    pub certificate: Vec<u8>,
}

fn decode<'a, T: serde::de::DeserializeOwned>(
    bytes: &'a [u8],
    maximum: usize,
) -> Result<Packet<'a, T>, Error> {
    if bytes.len() > maximum {
        return Err(Error::ByteLimit);
    }
    let mut decoder = serde_cbor::Deserializer::from_slice(bytes);
    let value = wire::map(&mut decoder).map_err(|_| Error::Packet)?;
    decoder.end().map_err(|_| Error::Packet)?;
    Ok(Packet {
        original: bytes,
        value,
    })
}

/// Admit exactly one call content map, retaining its original bytes.
///
/// # Errors
/// Refuses excess bytes, non-call content and non-strict CBOR fields.
pub fn decode_call_content(bytes: &[u8], maximum: usize) -> Result<Packet<'_, Content>, Error> {
    let packet = decode(bytes, maximum)?;
    if !matches!(packet.value, Content::Call { .. }) {
        return Err(Error::Kind);
    }
    Ok(packet)
}

/// Admit exactly one read-state content map.
///
/// # Errors
/// Refuses excess bytes, non-read-state content and non-strict CBOR fields.
pub fn decode_read_state_content(
    bytes: &[u8],
    maximum: usize,
) -> Result<Packet<'_, Content>, Error> {
    let packet = decode(bytes, maximum)?;
    if !matches!(packet.value, Content::ReadState { .. }) {
        return Err(Error::Kind);
    }
    Ok(packet)
}

/// Admit one authentication envelope without verifying it.
///
/// # Errors
/// Refuses excess bytes and malformed, unknown, duplicate or trailing fields.
pub fn decode_envelope(bytes: &[u8], maximum: usize) -> Result<Packet<'_, Envelope>, Error> {
    decode(bytes, maximum)
}

/// Admit one synchronous call response without authenticating its certificate.
///
/// # Errors
/// Refuses excess bytes and malformed, unknown, duplicate or trailing fields.
pub fn decode_call_response(
    bytes: &[u8],
    maximum: usize,
) -> Result<Packet<'_, CallResponse>, Error> {
    decode(bytes, maximum)
}

/// Admit one read-state response without authenticating its certificate.
///
/// # Errors
/// Refuses excess bytes and malformed, unknown, duplicate or trailing fields.
pub fn decode_read_state_response(
    bytes: &[u8],
    maximum: usize,
) -> Result<Packet<'_, ReadStateResponse>, Error> {
    decode(bytes, maximum)
}
