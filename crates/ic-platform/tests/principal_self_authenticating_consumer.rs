//! Explicit upstream self-authentication without selecting Candid or runtime.
use ic_platform::Principal;

/// Delegate to the registry implementation through the explicit capability.
pub fn self_authenticating(bytes: &[u8]) -> Principal {
    Principal::self_authenticating(bytes)
}

#[test]
fn upstream_self_authentication_is_unchanged() {
    use ic_platform::principal::ExternalPrincipal;
    for bytes in [&[][..], b"public key bytes", &[42; 1024]] {
        assert_eq!(
            ExternalPrincipal::from(self_authenticating(bytes)),
            ExternalPrincipal::self_authenticating(bytes)
        );
    }
}
