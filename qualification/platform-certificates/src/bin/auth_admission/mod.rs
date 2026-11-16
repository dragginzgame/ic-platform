//! Independently authored consumer adapter, not Auth implementation or authority.
//!
//! Only the issuer-signature component is modeled. Complete claims admission,
//! root authorization, receiver budgets and Canic endpoints remain outside it.
use candidate_platform::certificate::{
    CanisterSignatureKey, CanisterSignatureLimits, CanisterSignatureVerifyError,
    CertificateDecodeLimits, PreparedRootKey, TimePolicy, TimeWindow, VerificationLimits,
    VerifyError, authenticate_canister_signature, decode_canister_signature,
};
use candidate_types::Principal;
use sha2::{Digest, Sha256};

pub(super) mod tests;

// An explicit consumer compatibility choice: preserve Auth's lack of delegation
// freshness policy. Auth applies its own signing freshness after authentication.
const ALL_TIMES: TimeWindow = TimeWindow {
    reference_ns: 0,
    max_age_ns: 0,
    max_future_skew_ns: u64::MAX,
};

struct Policy<'a> {
    issuer: Principal,
    seed_hash: [u8; 32],
    root_raw: &'a [u8],
    now: u64,
    maximum_age: u64,
    signature_bytes: usize,
    verification: VerificationLimits,
}

#[derive(Debug, PartialEq, Eq)]
enum Refusal {
    SignatureSize,
    Key,
    Issuer,
    Seed,
    RootLength,
    Authentication,
    TimeEncoding,
    Future,
    Stale,
}

fn verify(
    digest: [u8; 32],
    signature: &[u8],
    key_der: &[u8],
    policy: &Policy<'_>,
) -> Result<u64, Refusal> {
    if signature.len() > policy.signature_bytes.min(131_072) {
        return Err(Refusal::SignatureSize);
    }
    let key = CanisterSignatureKey::from_der(key_der).map_err(|_| Refusal::Key)?;
    // Protected policy, not the proof's key, selects the approved identity.
    if key.issuer() != policy.issuer {
        return Err(Refusal::Issuer);
    }
    if <[u8; 32]>::from(Sha256::digest(key.seed())) != policy.seed_hash {
        return Err(Refusal::Seed);
    }
    let root = prepare_root(policy.root_raw)?;
    let message = message(digest);
    let decoded = decode_canister_signature(
        signature,
        CertificateDecodeLimits {
            bytes: policy.signature_bytes.min(131_072),
        },
    )
    .map_err(|_| Refusal::Authentication)?;
    let evidence = authenticate_canister_signature(
        decoded,
        &message,
        key,
        &root,
        TimePolicy {
            certificate: ALL_TIMES,
            delegation: ALL_TIMES,
        },
        CanisterSignatureLimits {
            message_bytes: message.len(),
            certificate_bytes: 131_072,
            verification: policy.verification,
        },
    )
    .map_err(|failure| match failure {
        CanisterSignatureVerifyError::Verification(VerifyError::InvalidTime) => {
            Refusal::TimeEncoding
        }
        _ => Refusal::Authentication,
    })?;
    let timestamp = evidence.time_ns();
    match policy.now.checked_sub(timestamp) {
        None => Err(Refusal::Future),
        Some(age) if age > policy.maximum_age => Err(Refusal::Stale),
        Some(_) => Ok(timestamp),
    }
}

fn message(digest: [u8; 32]) -> [u8; 60] {
    // External Auth protocol bytes: one domain-length byte, domain, claims digest.
    let mut result = [0; 60];
    result[..28].copy_from_slice(b"\x1bic-auth-portable-credential");
    result[28..].copy_from_slice(&digest);
    result
}

fn prepare_root(raw: &[u8]) -> Result<PreparedRootKey, Refusal> {
    // Fixed IC BLS SPKI framing, not an ASN.1 parser or trust-root discovery.
    const PREFIX: [u8; 37] = [
        0x30, 0x81, 0x82, 0x30, 0x1d, 0x06, 0x0d, 0x2b, 0x06, 0x01, 0x04, 0x01, 0x82, 0xdc, 0x7c,
        0x05, 0x03, 0x01, 0x02, 0x01, 0x06, 0x0c, 0x2b, 0x06, 0x01, 0x04, 0x01, 0x82, 0xdc, 0x7c,
        0x05, 0x03, 0x02, 0x01, 0x03, 0x61, 0x00,
    ];
    if raw.len() != 96 {
        return Err(Refusal::RootLength);
    }
    let mut der = [0; 133];
    der[..37].copy_from_slice(&PREFIX);
    der[37..].copy_from_slice(raw);
    PreparedRootKey::from_der(&der).map_err(|_| Refusal::Authentication)
}
