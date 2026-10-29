//! Opt-in registry identity conversion, without a codec or self-authentication.
use ic_platform::{Principal, principal::ExternalPrincipal};

/// An external identity is built directly from the owned significant bytes.
pub fn external(owned: Principal) -> ExternalPrincipal {
    owned.into()
}

#[test]
fn owned_identity_roundtrips_at_external_boundaries() {
    for len in 0..=29 {
        let owned = Principal::from_slice(&[2; 29][..len]);
        let registry = external(owned);
        assert_eq!(registry.as_slice(), owned.as_slice());
        assert_eq!(Principal::from(registry), owned);
        assert_eq!(Principal::from_text(owned.to_text()).unwrap(), owned);
    }
    assert!(Principal::try_from_slice(&[0; 30]).is_err());
}
