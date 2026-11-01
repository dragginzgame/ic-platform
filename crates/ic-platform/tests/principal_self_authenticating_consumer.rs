//! Explicit upstream self-authentication without selecting Candid or runtime.
use ic_platform::Principal;

/// Use the unchanged imported derivation through the explicit capability.
pub fn self_authenticating(bytes: &[u8]) -> Principal {
    Principal::self_authenticating(bytes)
}

#[test]
fn upstream_self_authentication_is_unchanged() {
    let expected = [
        209, 74, 2, 140, 42, 58, 43, 201, 71, 97, 2, 187, 40, 130, 52, 196, 21, 162, 176, 31, 130,
        142, 166, 42, 197, 179, 228, 47, 2,
    ];
    assert_eq!(self_authenticating(&[]).as_slice(), expected);
}
