//! Canister-key/witness composition; DER, hashing and BLS primitives stay upstream.

#[cfg(test)]
mod tests;

use super::{
    Authority, PreparedRootKey, Principal, TimePolicy, TimeWindow, VerificationLimits, VerifyError,
    forest, verify_certificate,
};
use crate::certificate::{
    CertificateDecodeLimits, DecodeError, DecodedCanisterSignature, decode_certificate,
};
use ic_certification::LookupResult;
use sha2::{Digest, Sha256};
use spki::{ObjectIdentifier, SubjectPublicKeyInfoRef, der::Decode};

/// Admitted short-form canister public key; admission does not authorize its issuer.
#[derive(Clone, Copy, Debug)]
pub struct CanisterSignatureKey<'a> {
    original: &'a [u8],
    issuer: Principal,
    seed: &'a [u8],
}
impl<'a> CanisterSignatureKey<'a> {
    /// Parse canonical, byte-aligned DER with the canister-signature OID and no parameters.
    /// Supports the retained 20–129 byte short-form envelope; larger keys refuse.
    ///
    /// # Errors
    /// Returns `InvalidKey` for framing, OID, bit alignment or Principal failures.
    pub fn from_der(bytes: &'a [u8]) -> Result<Self, CanisterSignatureVerifyError> {
        use CanisterSignatureVerifyError::InvalidKey;
        if !(20..=129).contains(&bytes.len()) {
            return Err(InvalidKey);
        }
        let key = SubjectPublicKeyInfoRef::from_der(bytes).map_err(|_| InvalidKey)?;
        if key.algorithm.oid != ObjectIdentifier::new_unwrap("1.3.6.1.4.1.56387.1.2")
            || key.algorithm.parameters.is_some()
        {
            return Err(InvalidKey);
        }
        let raw = key.subject_public_key.as_bytes().ok_or(InvalidKey)?;
        let (&length, payload) = raw.split_first().ok_or(InvalidKey)?;
        let issuer =
            Principal::try_from_slice(payload.get(..usize::from(length)).ok_or(InvalidKey)?)
                .map_err(|_| InvalidKey)?;
        Ok(Self {
            original: bytes,
            issuer,
            seed: &payload[usize::from(length)..],
        })
    }

    /// Exact original admitted DER.
    #[must_use]
    pub const fn as_der(&self) -> &'a [u8] {
        self.original
    }

    /// Issuer named by the key, not an application-approved identity.
    #[must_use]
    pub const fn issuer(&self) -> Principal {
        self.issuer
    }

    /// Opaque seed retained without normalization.
    #[must_use]
    pub const fn seed(&self) -> &'a [u8] {
        self.seed
    }
}

/// Additional input bounds for signature authentication after envelope decoding.
#[derive(Clone, Copy, Debug)]
pub struct CanisterSignatureLimits {
    /// Maximum complete message length, checked before hashing.
    pub message_bytes: usize,
    /// Maximum embedded signing-certificate bytes, checked before decoding it.
    pub certificate_bytes: usize,
    /// Shared node budget across witness, signing and delegation trees; depth
    /// applies to each tree. Delegation bytes and range-pair limits are unchanged.
    pub verification: VerificationLimits,
}

/// Refusal without partially authenticated signature evidence.
#[derive(Debug, thiserror::Error)]
pub enum CanisterSignatureVerifyError {
    /// Invalid or unsupported canister-signature DER key.
    #[error("invalid canister-signature key")]
    InvalidKey,
    /// Caller-selected message bound exceeded before hashing.
    #[error("canister-signature message exceeds allowance")]
    MessageLimit,
    /// Embedded certificate lacks its required self-describing CBOR tag.
    #[error("canister-signature certificate requires CBOR tag 55799")]
    MissingCertificateTag,
    /// Embedded certificate input or grammar refusal.
    #[error("invalid embedded certificate: {0}")]
    Decode(#[from] DecodeError),
    /// Certificate authority, signature, time, or shared tree admission refusal.
    #[error("canister-signature certificate or tree refused: {0}")]
    Verification(#[from] VerifyError),
    /// Required message/seed witness path is missing, pruned, non-leaf or nonempty.
    #[error("canister-signature witness does not prove the message and seed")]
    Witness,
    /// Issuer's certified data does not equal the witness digest.
    #[error("canister-signature witness is not bound to certified data")]
    CertifiedData,
}

/// Immutable signature evidence for the exact message, key, root and time context.
/// Application signer/seed authorization and domain selection remain caller-owned.
///
/// ```compile_fail
/// use ic_platform::certificate::AuthenticatedCanisterSignature;
/// fn mutate(value: &mut AuthenticatedCanisterSignature<'_>) {
///     value.message()[0] = 0;
/// }
/// ```
/// ```compile_fail
/// use ic_platform::certificate::AuthenticatedCanisterSignature;
/// fn fabricate(bytes: &[u8]) -> AuthenticatedCanisterSignature<'_> {
///     serde_cbor::from_slice(bytes).unwrap()
/// }
/// ```
#[derive(Debug)]
pub struct AuthenticatedCanisterSignature<'a> {
    decoded: DecodedCanisterSignature<'a>,
    message: &'a [u8],
    key: CanisterSignatureKey<'a>,
    root: [u8; 133],
    window: TimeWindow,
    time_ns: u64,
    delegation: Option<(Principal, TimeWindow, u64)>,
}
impl<'a> AuthenticatedCanisterSignature<'a> {
    /// Exact original outer signature bytes.
    #[must_use]
    pub const fn original_bytes(&self) -> &'a [u8] {
        self.decoded.original_bytes()
    }

    /// Exact signed message, including any caller-selected domain prefix.
    #[must_use]
    pub const fn message(&self) -> &'a [u8] {
        self.message
    }

    /// Admitted key bound to this signature.
    #[must_use]
    pub const fn key(&self) -> CanisterSignatureKey<'a> {
        self.key
    }

    /// Explicit trust root used for this verification.
    #[must_use]
    pub const fn root_key_der(&self) -> &[u8; 133] {
        &self.root
    }

    /// Signing-certificate time in Unix nanoseconds, never delegation time.
    #[must_use]
    pub const fn time_ns(&self) -> u64 {
        self.time_ns
    }

    /// Inclusive signing-time window checked by the verifier.
    #[must_use]
    pub const fn time_window(&self) -> TimeWindow {
        self.window
    }

    /// Subnet, separate time window and timestamp of an authenticated delegation.
    #[must_use]
    pub const fn delegation(&self) -> Option<(Principal, TimeWindow, u64)> {
        self.delegation
    }
}

/// Authenticate a decoded canister signature under explicit root/time/resource inputs.
///
/// Checks ordered witness grammar, the empty seed/message leaf, certified-data
/// binding and the shared certificate verifier. Delegations require certified
/// issuer ranges and a known non-cloud-engine subnet type. No signer/seed policy,
/// clock, network root or message-domain prefix is selected by this function.
///
/// # Errors
/// Refuses oversized input, malformed/tagless certificates, invalid witness or
/// data binding, exhausted bounds, or certificate authority/key/signature/time failures.
pub fn authenticate_canister_signature<'a>(
    decoded: DecodedCanisterSignature<'a>,
    message: &'a [u8],
    key: CanisterSignatureKey<'a>,
    root: &PreparedRootKey,
    time: TimePolicy,
    limits: CanisterSignatureLimits,
) -> Result<AuthenticatedCanisterSignature<'a>, CanisterSignatureVerifyError> {
    if message.len() > limits.message_bytes {
        return Err(CanisterSignatureVerifyError::MessageLimit);
    }
    let certificate_bytes = decoded.certificate_bytes();
    if certificate_bytes.len() > limits.certificate_bytes {
        return Err(DecodeError::InputLimit {
            actual: certificate_bytes.len(),
            maximum: limits.certificate_bytes,
        }
        .into());
    }
    if !certificate_bytes.starts_with(&[0xd9, 0xd9, 0xf7]) {
        return Err(CanisterSignatureVerifyError::MissingCertificateTag);
    }
    let mut verification = limits.verification;
    forest(
        decoded.unverified_witness().as_ref(),
        &mut verification.tree_nodes,
        verification.tree_depth,
        &mut None,
        &mut false,
    )?;
    let seed_hash: [u8; 32] = Sha256::digest(key.seed).into();
    let message_hash: [u8; 32] = Sha256::digest(message).into();
    if !matches!(
        decoded
            .unverified_witness()
            .lookup_path([b"sig".as_slice(), &seed_hash, &message_hash]),
        LookupResult::Found([])
    ) {
        return Err(CanisterSignatureVerifyError::Witness);
    }
    let certificate = decode_certificate(
        certificate_bytes,
        CertificateDecodeLimits {
            bytes: limits.certificate_bytes,
        },
    )?;
    let verified = verify_certificate(
        certificate.unverified(),
        root,
        Authority::CanisterSignature(key.issuer),
        time,
        verification,
    )?;
    let digest = decoded.unverified_witness().digest();
    if !matches!(certificate.unverified().tree.lookup_path([b"canister".as_slice(), key.issuer.as_slice(), b"certified_data"]), LookupResult::Found(value) if value == digest)
    {
        return Err(CanisterSignatureVerifyError::CertifiedData);
    }
    Ok(AuthenticatedCanisterSignature {
        decoded,
        message,
        key,
        root: *root.as_der(),
        window: time.certificate,
        time_ns: verified.time_ns,
        delegation: verified.delegation,
    })
}
