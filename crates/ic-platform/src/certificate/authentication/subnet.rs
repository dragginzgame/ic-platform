//! A narrow authenticated signing-time projection for a caller-selected subnet.

use super::{
    Authority, DecodedCertificate, PreparedRootKey, Principal, TimePolicy, TimeWindow,
    VerificationLimits, VerifyError, verify_certificate,
};

/// Immutable subnet signing time bound to the original response certificate.
///
/// This is evidence for the supplied time window, not continuous uptime, current
/// replica version or maintenance completion. No canister-range authority is claimed.
///
/// ```compile_fail
/// use ic_platform::certificate::AuthenticatedSubnetTime;
/// fn change(value: &mut AuthenticatedSubnetTime<'_>) { value.time_ns = 0; }
/// ```
/// ```compile_fail
/// use ic_platform::certificate::AuthenticatedSubnetTime;
/// fn fabricate(bytes: &[u8]) -> AuthenticatedSubnetTime<'_> {
///     serde_cbor::from_slice(bytes).unwrap()
/// }
/// ```
#[derive(Debug)]
pub struct AuthenticatedSubnetTime<'a> {
    source: Source<'a>,
    root_der: [u8; 133],
    subnet: Principal,
    time_ns: u64,
    window: TimeWindow,
    delegation: Option<(TimeWindow, u64)>,
}
impl<'a> AuthenticatedSubnetTime<'a> {
    /// Exact original bytes for the encoded entry point; None for parsed input.
    /// Original encoding cannot be recovered from an upstream parsed certificate.
    #[must_use]
    pub const fn original_bytes(&self) -> Option<&'a [u8]> {
        match self.source {
            Source::Encoded(bytes) => Some(bytes),
            Source::Parsed(_) => None,
        }
    }

    /// The exact borrowed upstream certificate when using the parsed entry point.
    #[must_use]
    pub const fn certificate(&self) -> Option<&'a ic_certification::Certificate> {
        match self.source {
            Source::Parsed(value) => Some(value),
            Source::Encoded(_) => None,
        }
    }

    /// Explicit trusted root selected for this verification.
    #[must_use]
    pub const fn root_key_der(&self) -> &[u8; 133] {
        &self.root_der
    }

    /// Requested subnet, bound to delegated authority or the trusted root-subnet input.
    #[must_use]
    pub const fn subnet(&self) -> Principal {
        self.subnet
    }

    /// Raw signing-certificate Unix nanoseconds, never the delegation timestamp.
    #[must_use]
    pub const fn time_ns(&self) -> u64 {
        self.time_ns
    }

    /// Explicit signing-certificate window checked by the verifier.
    #[must_use]
    pub const fn time_window(&self) -> TimeWindow {
        self.window
    }

    /// Delegation window and timestamp, if delegated; separate from signing time.
    #[must_use]
    pub const fn delegation(&self) -> Option<(TimeWindow, u64)> {
        self.delegation
    }
}

/// Authenticate signing time for exactly the requested subnet.
///
/// Delegated certificates must name `requested_subnet`; both signatures, checked
/// keys, tree bounds and separate time windows are verified. Subnet observations
/// require subnet identity, not a canister-range match.
///
/// For a certificate without delegation, `trusted_root_subnet` must be supplied
/// and equal `requested_subnet`. This is an additional caller-owned trust input:
/// it must come from trusted network configuration binding that subnet to `root`,
/// never from the response, endpoint routing or simply copying the requested ID.
/// A root signature alone does not identify an arbitrary child subnet. Supply
/// `None` when no trusted root-subnet binding is available; direct roots then refuse.
///
/// The caller supplies live freshness windows and retains original response bytes.
/// This function selects no clock, endpoint, retries or application acceptance.
/// Complete u64 LEB128 time is required; nonminimal encodings remain accepted.
/// Decode byte limits apply before this call; node/depth and delegation byte bounds
/// apply here. `canister_ranges` is unused for this subnet-specific contract.
///
/// # Errors
/// Returns typed root-binding/subnet refusals or the shared strict certificate,
/// delegation, tree, key, signature, resource and time refusals.
pub fn authenticate_subnet_time<'a>(
    decoded: DecodedCertificate<'a>,
    root: &PreparedRootKey,
    requested_subnet: Principal,
    trusted_root_subnet: Option<Principal>,
    time: TimePolicy,
    limits: VerificationLimits,
) -> Result<AuthenticatedSubnetTime<'a>, VerifyError> {
    let verified = verify_certificate(
        &decoded.certificate,
        root,
        Authority::Subnet {
            requested: requested_subnet,
            root_subnet: trusted_root_subnet,
        },
        time,
        limits,
    )?;
    let original = decoded.original_bytes();
    // The narrow projection retains original bytes, not the allocated decoded tree.
    drop(decoded);
    Ok(AuthenticatedSubnetTime {
        source: Source::Encoded(original),
        root_der: *root.as_der(),
        subnet: requested_subnet,
        time_ns: verified.time_ns,
        window: time.certificate,
        delegation: verified.delegation.map(|(_, window, time)| (window, time)),
    })
}

#[derive(Debug)]
enum Source<'a> {
    Encoded(&'a [u8]),
    Parsed(&'a ic_certification::Certificate),
}

/// Authenticate a parsed certificate returned by the upstream native Agent.
///
/// This borrows the exact upstream certificate and repeats signature, subnet,
/// key, tree-bound and explicit time checks. It neither trusts an earlier success
/// flag nor re-encodes data to manufacture original-byte evidence. The result's
/// `original_bytes()` is None; `certificate()` retains this immutable borrow.
///
/// Root-subnet trust admission is identical to [authenticate_subnet_time]. The
/// caller owns HTTP/body bounds and upstream outer decoding: this API cannot
/// establish original CBOR grammar, duplicate-field refusal or preallocation
/// limits for an already parsed object. Tree nodes/depth and embedded delegation
/// byte limits still apply before hashing; embedded delegation decoding is strict.
/// No Agent dependency, transport or caller-selected freshness default is added.
///
/// ```compile_fail
/// use ic_platform::certificate::AuthenticatedSubnetTime;
/// fn mutate(value: &AuthenticatedSubnetTime<'_>) {
///     value.certificate().unwrap().signature.clear();
/// }
/// ```
///
/// # Errors
/// Returns the same root/subnet, key/signature, traversal and time refusals as
/// the encoded entry point. It does not report outer encoding-byte refusals.
pub fn authenticate_subnet_time_from_certificate<'a>(
    certificate: &'a ic_certification::Certificate,
    root: &PreparedRootKey,
    requested_subnet: Principal,
    trusted_root_subnet: Option<Principal>,
    time: TimePolicy,
    limits: VerificationLimits,
) -> Result<AuthenticatedSubnetTime<'a>, VerifyError> {
    let verified = verify_certificate(
        certificate,
        root,
        Authority::Subnet {
            requested: requested_subnet,
            root_subnet: trusted_root_subnet,
        },
        time,
        limits,
    )?;
    Ok(AuthenticatedSubnetTime {
        source: Source::Parsed(certificate),
        root_der: *root.as_der(),
        subnet: requested_subnet,
        time_ns: verified.time_ns,
        window: time.certificate,
        delegation: verified.delegation.map(|(_, window, time)| (window, time)),
    })
}
