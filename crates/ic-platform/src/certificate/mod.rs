//! Strict decoding of untrusted certificate bytes, without authentication.

#[cfg(feature = "certificate-authentication")]
mod authentication;
mod wire;
#[cfg(feature = "certificate-authentication")]
pub use authentication::{
    AuthenticatedCertificate, AuthenticatedSubnetTime, PreparedRootKey, TimePolicy, TimeWindow,
    VerificationLimits, VerifyError, authenticate_certificate, authenticate_direct_certificate,
    authenticate_subnet_time, authenticate_subnet_time_from_certificate,
};

/// Explicit input bound, checked before decoding or allocating decoded fields.
/// This is not a decoder heap/work quota; CBOR retains its own recursion limit.
#[derive(Clone, Copy, Debug)]
pub struct CertificateDecodeLimits {
    /// Maximum complete encoded certificate length, inclusive.
    pub bytes: usize,
}

/// Refusal before any decoded certificate is returned.
#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    /// Complete input exceeds the caller's byte allowance.
    #[error("certificate input has {actual} bytes, exceeding {maximum}")]
    InputLimit {
        /// Supplied byte length.
        actual: usize,
        /// Caller-selected allowance.
        maximum: usize,
    },
    /// Invalid certificate grammar, including malformed CBOR or trailing data.
    #[error("invalid certificate encoding: {0}")]
    InvalidEncoding(#[source] serde_cbor::Error),
}

/// Unauthenticated decoded data bound to its exact original bytes.
///
/// Private fields and immutable access do not confer authentication. No signature,
/// key, delegation, range, ordering or authenticated-time checks have occurred.
///
/// Consumers cannot replace decoded fields:
/// ```compile_fail
/// use ic_platform::certificate::DecodedCertificate;
/// fn edit(value: &mut DecodedCertificate<'_>) {
///     value.unverified().signature.clear();
/// }
/// ```
/// Nor can they fabricate a wrapper around unrelated original bytes:
/// ```compile_fail
/// use ic_platform::certificate::DecodedCertificate;
/// fn fabricate(certificate: ic_certification::Certificate) {
///     let _ = DecodedCertificate { original: &[], certificate };
/// }
/// ```
#[derive(Debug)]
pub struct DecodedCertificate<'a> {
    original: &'a [u8],
    certificate: ic_certification::Certificate,
}
impl<'a> DecodedCertificate<'a> {
    /// Borrows the complete supplied encoding, without re-encoding or normalization.
    #[must_use]
    pub const fn original_bytes(&self) -> &'a [u8] {
        self.original
    }

    /// Views untrusted upstream certificate values. This is not verified evidence.
    #[must_use]
    pub const fn unverified(&self) -> &ic_certification::Certificate {
        &self.certificate
    }
}

/// Decode exactly one certificate after admitting its complete input byte length.
///
/// Fields reject duplicates and unknown names. Tree tags/arity and byte-string
/// payloads are strict; delegation certificate bytes stay opaque. No authentication
/// is performed. The caller keeps original byte custody for the returned lifetime.
///
/// # Errors
/// Returns input-limit or encoding refusal without a partial decoded value.
pub fn decode_certificate(
    bytes: &[u8],
    limits: CertificateDecodeLimits,
) -> Result<DecodedCertificate<'_>, DecodeError> {
    if bytes.len() > limits.bytes {
        return Err(DecodeError::InputLimit {
            actual: bytes.len(),
            maximum: limits.bytes,
        });
    }
    let certificate = wire::decode(bytes).map_err(DecodeError::InvalidEncoding)?;
    Ok(DecodedCertificate {
        original: bytes,
        certificate,
    })
}

#[cfg(test)]
mod tests;

#[cfg(feature = "canister-signature-decode")]
mod canister_signature;
#[cfg(feature = "canister-signature-decode")]
pub use canister_signature::{
    CanisterSignatureDecodeError, DecodedCanisterSignature, decode_canister_signature,
};

#[cfg(feature = "canister-signature-authentication")]
pub use authentication::{
    AuthenticatedCanisterSignature, CanisterSignatureKey, CanisterSignatureLimits,
    CanisterSignatureVerifyError, authenticate_canister_signature,
};
