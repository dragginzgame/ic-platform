//! Replay frozen replica-signed envelopes through the public native/Wasm API.
#![forbid(unsafe_code)]

use candidate_platform::certificate::{
    CanisterSignatureDecodeError, CanisterSignatureKey, CanisterSignatureLimits,
    CanisterSignatureVerifyError as Error, CertificateDecodeLimits, DecodeError, PreparedRootKey,
    TimePolicy, TimeWindow, VerificationLimits, VerifyError, authenticate_canister_signature,
    decode_canister_signature,
};
use candidate_types::Principal;
use ic_certification::{fork, labeled, leaf};
use serde_cbor::{Value, value::to_value};

const MESSAGE: &[u8] = b"ic-platform observed signature";
const DIRECT_KEY: &[u8] = include_bytes!("../../fixtures/signatures/direct.der");
const DELEGATED_KEY: &[u8] = include_bytes!("../../fixtures/signatures/delegated.der");

fn main() {
    let root =
        PreparedRootKey::from_der(include_bytes!("../../fixtures/signatures/root.der")).unwrap();
    let wrong_root =
        PreparedRootKey::from_der(include_bytes!("../../fixtures/mainnet-root.der")).unwrap();
    for (bytes, key_bytes, issuer, timestamp, delegated) in [
        (
            include_bytes!("../../fixtures/signatures/direct.cbor").as_slice(),
            DIRECT_KEY,
            "xp3jw-ot777-77777-aaaaa-cai",
            1_620_328_630_000_000_005,
            false,
        ),
        (
            include_bytes!("../../fixtures/signatures/delegated.cbor").as_slice(),
            DELEGATED_KEY,
            "dllsh-pd777-77776-qaaaa-cai",
            1_620_328_630_000_000_008,
            true,
        ),
    ] {
        let bytes = std::hint::black_box(bytes);
        let key = CanisterSignatureKey::from_der(key_bytes).unwrap();
        let decoded = || {
            decode_canister_signature(bytes, CertificateDecodeLimits { bytes: bytes.len() })
                .unwrap()
        };
        let window = TimeWindow {
            reference_ns: timestamp,
            max_age_ns: 0,
            max_future_skew_ns: 0,
        };
        let time = TimePolicy {
            certificate: window,
            delegation: window,
        };
        let limits = CanisterSignatureLimits {
            message_bytes: MESSAGE.len(),
            certificate_bytes: decoded().certificate_bytes().len(),
            verification: VerificationLimits {
                tree_nodes: 4096,
                tree_depth: 64,
                delegation_bytes: 131072,
                canister_ranges: 256,
            },
        };
        let evidence =
            authenticate_canister_signature(decoded(), MESSAGE, key, &root, time, limits).unwrap();
        assert_eq!(evidence.time_ns(), timestamp);
        assert_eq!(evidence.original_bytes().as_ptr(), bytes.as_ptr());
        assert_eq!(evidence.message().as_ptr(), MESSAGE.as_ptr());
        assert_eq!(evidence.key().as_der().as_ptr(), key_bytes.as_ptr());
        assert_eq!(
            evidence.key().issuer(),
            Principal::from_text(issuer).unwrap()
        );
        assert_eq!(evidence.key().seed(), b"qualification-seed");
        assert_eq!(evidence.root_key_der(), root.as_der());
        assert_eq!(evidence.time_window(), window);
        assert_eq!(evidence.delegation().is_some(), delegated);
        if let Some((subnet, delegated_window, delegated_time)) = evidence.delegation() {
            assert_eq!(
                subnet,
                Principal::from_text(
                    "sgier-p6fyz-ejpwm-jghgc-4ypgv-6ft53-5hns6-ojhae-wvlic-ifcah-eqe"
                )
                .unwrap()
            );
            assert_eq!(delegated_window, window);
            assert_eq!(delegated_time, timestamp);
        }
        assert!(matches!(
            authenticate_canister_signature(decoded(), b"changed", key, &root, time, limits),
            Err(Error::Witness)
        ));
        let mut changed_seed = key_bytes.to_vec();
        *changed_seed.last_mut().unwrap() ^= 1;
        assert!(matches!(
            authenticate_canister_signature(
                decoded(),
                MESSAGE,
                CanisterSignatureKey::from_der(&changed_seed).unwrap(),
                &root,
                time,
                limits
            ),
            Err(Error::Witness)
        ));
        assert!(matches!(
            authenticate_canister_signature(decoded(), MESSAGE, key, &wrong_root, time, limits),
            Err(Error::Verification(VerifyError::Signature))
        ));
        let wrong_issuer =
            CanisterSignatureKey::from_der(if delegated { DIRECT_KEY } else { DELEGATED_KEY })
                .unwrap();
        let refused =
            authenticate_canister_signature(decoded(), MESSAGE, wrong_issuer, &root, time, limits)
                .unwrap_err();
        if delegated {
            assert!(matches!(
                refused,
                Error::Verification(VerifyError::NotAuthorized)
            ));
        } else {
            assert!(matches!(refused, Error::CertifiedData));
        }
        let mut stale = time;
        stale.certificate.reference_ns += 1;
        assert!(matches!(
            authenticate_canister_signature(decoded(), MESSAGE, key, &root, stale, limits),
            Err(Error::Verification(VerifyError::TimeOutsideWindow))
        ));
        let mut future = time;
        future.certificate.reference_ns -= 1;
        assert!(matches!(
            authenticate_canister_signature(decoded(), MESSAGE, key, &root, future, limits),
            Err(Error::Verification(VerifyError::TimeOutsideWindow))
        ));
        let mut delegation_time = time;
        delegation_time.delegation.reference_ns += 1;
        let result = authenticate_canister_signature(
            decoded(),
            MESSAGE,
            key,
            &root,
            delegation_time,
            limits,
        );
        if delegated {
            assert!(matches!(
                result,
                Err(Error::Verification(VerifyError::TimeOutsideWindow))
            ));
        } else {
            assert!(result.is_ok());
        }
        let mut bounded = limits;
        bounded.message_bytes -= 1;
        assert!(matches!(
            authenticate_canister_signature(decoded(), MESSAGE, key, &root, time, bounded),
            Err(Error::MessageLimit)
        ));
        bounded = limits;
        bounded.certificate_bytes -= 1;
        assert!(matches!(
            authenticate_canister_signature(decoded(), MESSAGE, key, &root, time, bounded),
            Err(Error::Decode(DecodeError::InputLimit { .. }))
        ));
        bounded = limits;
        bounded.verification.tree_nodes = 0;
        assert!(matches!(
            authenticate_canister_signature(decoded(), MESSAGE, key, &root, time, bounded),
            Err(Error::Verification(VerifyError::TreeLimit))
        ));
        bounded = limits;
        bounded.verification.tree_depth = 0;
        assert!(matches!(
            authenticate_canister_signature(decoded(), MESSAGE, key, &root, time, bounded),
            Err(Error::Verification(VerifyError::TreeLimit))
        ));
        if delegated {
            bounded = limits;
            bounded.verification.delegation_bytes = 0;
            assert!(matches!(
                authenticate_canister_signature(decoded(), MESSAGE, key, &root, time, bounded),
                Err(Error::Verification(VerifyError::DelegationLimit))
            ));
            bounded = limits;
            bounded.verification.canister_ranges = 0;
            assert!(matches!(
                authenticate_canister_signature(decoded(), MESSAGE, key, &root, time, bounded),
                Err(Error::Verification(VerifyError::RangeLimit))
            ));
        }
        assert!(matches!(
            decode_canister_signature(
                bytes,
                CertificateDecodeLimits {
                    bytes: bytes.len() - 1
                }
            ),
            Err(CanisterSignatureDecodeError::InputLimit { .. })
        ));
        for end in 0..bytes.len() {
            assert!(
                decode_canister_signature(
                    &bytes[..end],
                    CertificateDecodeLimits { bytes: bytes.len() }
                )
                .is_err()
            );
        }
        let mut trailing = bytes.to_vec();
        trailing.push(0);
        assert!(
            decode_canister_signature(
                &trailing,
                CertificateDecodeLimits {
                    bytes: trailing.len()
                }
            )
            .is_err()
        );
        let corrupted = corrupt_certificate(bytes, false);
        assert!(matches!(
            authenticate_canister_signature(
                decode_canister_signature(
                    &corrupted,
                    CertificateDecodeLimits {
                        bytes: corrupted.len()
                    }
                )
                .unwrap(),
                MESSAGE,
                key,
                &root,
                time,
                limits
            ),
            Err(Error::Verification(VerifyError::Signature))
        ));
        if delegated {
            let corrupted = corrupt_certificate(bytes, true);
            assert!(matches!(
                authenticate_canister_signature(
                    decode_canister_signature(
                        &corrupted,
                        CertificateDecodeLimits {
                            bytes: corrupted.len()
                        }
                    )
                    .unwrap(),
                    MESSAGE,
                    key,
                    &root,
                    time,
                    limits
                ),
                Err(Error::Verification(VerifyError::Signature))
            ));
        }
        // Keep the correct message path but change the digest outside that path.
        let different_witness = fork(
            decoded().unverified_witness().clone(),
            labeled(b"zzz".to_vec(), leaf(Vec::new())),
        );
        let mut envelope: Value = serde_cbor::from_slice(bytes).unwrap();
        *field(&mut envelope, "tree") = to_value(different_witness).unwrap();
        let wrong_digest = tagged(&envelope);
        assert!(matches!(
            authenticate_canister_signature(
                decode_canister_signature(
                    &wrong_digest,
                    CertificateDecodeLimits {
                        bytes: wrong_digest.len()
                    }
                )
                .unwrap(),
                MESSAGE,
                key,
                &root,
                time,
                limits
            ),
            Err(Error::CertifiedData)
        ));
    }
}

// Fixture-only mutation uses the independent CBOR value API, never private
// candidate representations or its encoding/verification helpers.
fn field<'a>(value: &'a mut Value, key: &str) -> &'a mut Value {
    let Value::Map(map) = value else {
        panic!("fixture must be a map")
    };
    map.get_mut(&Value::Text(key.into())).unwrap()
}

fn tagged(value: &Value) -> Vec<u8> {
    let mut bytes = vec![0xd9, 0xd9, 0xf7];
    bytes.extend(serde_cbor::to_vec(value).unwrap());
    bytes
}

fn corrupt_certificate(bytes: &[u8], delegation: bool) -> Vec<u8> {
    let mut envelope: Value = serde_cbor::from_slice(bytes).unwrap();
    let Value::Bytes(encoded) = field(&mut envelope, "certificate") else {
        panic!("certificate bytes")
    };
    let mut certificate: Value = serde_cbor::from_slice(encoded).unwrap();
    if delegation {
        let Value::Bytes(delegated) = field(field(&mut certificate, "delegation"), "certificate")
        else {
            panic!("delegation bytes")
        };
        let mut inner: Value = serde_cbor::from_slice(delegated).unwrap();
        let Value::Bytes(signature) = field(&mut inner, "signature") else {
            panic!("signature bytes")
        };
        signature[0] ^= 1;
        *delegated = tagged(&inner);
    } else {
        let Value::Bytes(signature) = field(&mut certificate, "signature") else {
            panic!("signature bytes")
        };
        signature[0] ^= 1;
    }
    *encoded = tagged(&certificate);
    tagged(&envelope)
}
