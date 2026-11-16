//! Public Types crossing through the unchanged, pinned Auth passive model.
#![forbid(unsafe_code)]

use ic_auth_protocol_types::{AudienceId, CanonicalId, PortableClaims, PortableMechanism};
use ic_platform_types::{Account, Principal};

fn main() {
    // Assignment in both directions proves Cargo identity, not byte conversion.
    let issuer = Principal::from_slice(&[1, 2, 3]);
    let auth_issuer: ic_auth_protocol_types::Principal = issuer;
    let owner: Principal = auth_issuer;
    let account = Account {
        owner,
        subaccount: None,
    };
    let claims = PortableClaims {
        mechanism: PortableMechanism::Certified,
        audience: AudienceId {
            canonical_network_id: CanonicalId::from_bytes([1; 32]),
            fleet_id: CanonicalId::from_bytes([2; 32]),
        },
        issuer: account.owner,
        key_binding_id: [3; 32],
        browser_public_key: [4; 32],
        account_subject: Principal::from_slice(&[5, 6]),
        device_slot: 0,
        device_generation: 1,
        login_started_at_ns: 0,
        login_deadline_ns: u64::MAX,
        issued_at_ns: 1,
        admission_expires_at_ns: u64::MAX,
        nonce: [0; 16],
        grants: Vec::new(),
    };
    assert_eq!(claims.issuer, issuer);
    assert_eq!(claims.account_subject.as_slice(), [5, 6]);
    assert_eq!(claims.clone(), claims);
    let mut buffer = [0; Principal::MAX_LENGTH_IN_TEXT];
    assert_eq!(claims.issuer.write_text(&mut buffer), issuer.to_text());
    // Fixed upstream-independent SHA-224 answer for "abc", followed by IC tag 2.
    #[cfg(feature = "principal-self-authenticating")]
    {
        let expected = [
            0x23, 0x09, 0x7d, 0x22, 0x34, 0x05, 0xd8, 0x22, 0x86, 0x42, 0xa4, 0x77, 0xbd, 0xa2,
            0x55, 0xb3, 0x2a, 0xad, 0xbc, 0xe4, 0xbd, 0xa0, 0xb3, 0xf7, 0xe3, 0x6c, 0x9d, 0xa7,
            0x02,
        ];
        let derived: Principal = ic_auth_protocol_types::Principal::self_authenticating(b"abc");
        assert_eq!(derived.as_slice(), expected);
    }
}
