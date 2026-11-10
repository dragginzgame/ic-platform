//! Independent public-API differential tests and opt-in native timings.
use ic_canister_sig_creation as reference;
use ic_platform::{
    Principal, canister_sig_creation as owned, signature_verification as verification,
};
use std::{hint::black_box, time::Instant};

fn hex(text: &str) -> Vec<u8> {
    text.trim()
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

struct Case {
    name: &'static str,
    message: &'static [u8],
    signature: Vec<u8>,
    public: Vec<u8>,
    root: Vec<u8>,
}
fn cases() -> Vec<Case> {
    let issuer = Principal::from_slice(&[1; 10]);
    let public = owned::CanisterSigPublicKey::new(issuer, b"seed".to_vec()).to_der();
    [
        ("direct", include_str!("fixtures/signature-time-valid.hex")),
        (
            "delegated-legacy",
            include_str!("fixtures/signature-range-legacy-valid.hex"),
        ),
        (
            "delegated-sharded",
            include_str!("fixtures/signature-range-sharded-valid.hex"),
        ),
    ]
    .into_iter()
    .map(|(name, encoded)| Case {
        name,
        message: b"message",
        signature: hex(encoded),
        public: public.clone(),
        root: hex(include_str!("fixtures/bls-public.hex")),
    })
    .collect()
}

#[test]
fn key_and_hash_matrix_matches_public_reference() {
    assert_eq!(owned::IC_ROOT_PK_DER, reference::IC_ROOT_PK_DER);
    assert_eq!(
        owned::IC_ROOT_PK_DER_PREFIX,
        reference::IC_ROOT_PK_DER_PREFIX
    );
    for issuer_len in 0..=29 {
        let issuer: Vec<_> = (0..issuer_len).map(|i| (i * 7 + 3) as u8).collect();
        for seed_len in [
            0, 1, 16, 32, 77, 78, 79, 96, 107, 108, 109, 127, 128, 255, 256, 1024,
        ] {
            let seed: Vec<_> = (0..seed_len).map(|i| (i * 13 + 1) as u8).collect();
            let a = owned::CanisterSigPublicKey::new(Principal::from_slice(&issuer), seed.clone());
            let b =
                reference::CanisterSigPublicKey::new(issuer.as_slice().try_into().unwrap(), seed);
            assert_eq!(a.to_raw(), b.to_raw());
            assert_eq!(a.to_der(), b.to_der());
            assert_eq!(
                owned::CanisterSigPublicKey::try_from_raw(&b.to_raw()).unwrap(),
                a
            );
            let der = a.to_der();
            if der.len() <= 129 {
                assert_eq!(
                    owned::CanisterSigPublicKey::try_from(der.as_slice()).unwrap(),
                    a
                );
                assert_eq!(
                    reference::CanisterSigPublicKey::try_from(der.as_slice()).unwrap(),
                    b
                );
            } else {
                assert!(owned::CanisterSigPublicKey::try_from(der.as_slice()).is_err());
            }
        }
    }
    for domain_len in [0, 1, 13, 255, 256, 513] {
        let domain = vec![0x61; domain_len];
        for payload_len in [0, 1, 63, 64, 65, 4096] {
            let payload = vec![0xd3; payload_len];
            assert_eq!(owned::hash_bytes(&payload), reference::hash_bytes(&payload));
            assert_eq!(
                owned::hash_with_domain(&domain, &payload),
                reference::hash_with_domain(&domain, &payload)
            );
        }
    }
}

#[test]
fn malformed_raw_keys_and_roots_match_reference() {
    for len in 0..=64 {
        for first in [0, 1, 28, 29, 30, 64, 255] {
            let mut raw = vec![0xa7; len];
            if let Some(byte) = raw.first_mut() {
                *byte = first;
            }
            let a = owned::CanisterSigPublicKey::try_from_raw(&raw);
            let b = reference::CanisterSigPublicKey::try_from_raw(&raw);
            assert_eq!(a.is_ok(), b.is_ok(), "raw len={len} first={first}");
            if let (Err(a), Err(b)) = (a, b) {
                assert_eq!(a, b);
            }
        }
    }
    for len in 0..=150 {
        let mut root = owned::IC_ROOT_PK_DER.to_vec();
        root.resize(len, 0);
        assert_eq!(
            owned::extract_raw_root_pk_from_der(&root),
            reference::extract_raw_root_pk_from_der(&root)
        );
    }
    for index in 0..37 {
        let mut root = owned::IC_ROOT_PK_DER.to_vec();
        root[index] ^= 0x80;
        assert_eq!(
            owned::extract_raw_root_pk_from_der(&root),
            reference::extract_raw_root_pk_from_der(&root)
        );
    }
}

#[test]
fn signature_mutations_match_reference() {
    for case in cases() {
        assert_eq!(
            verification::verify_canister_sig(
                case.message,
                &case.signature,
                &case.public,
                &case.root
            ),
            Ok(())
        );
        assert_eq!(
            ic_signature_verification::verify_canister_sig(
                case.message,
                &case.signature,
                &case.public,
                &case.root
            ),
            Ok(())
        );
        assert_eq!(
            verification::verify_canister_sig_with_time(
                case.message,
                &case.signature,
                &case.public,
                &case.root
            ),
            Ok(if case.name == "direct" { 128 } else { 42 })
        );
        for index in (0..case.signature.len()).step_by(7) {
            let mut changed = case.signature.clone();
            changed[index] ^= 0x04;
            let a =
                verification::verify_canister_sig(case.message, &changed, &case.public, &case.root)
                    .is_ok();
            let b = ic_signature_verification::verify_canister_sig(
                case.message,
                &changed,
                &case.public,
                &case.root,
            )
            .is_ok();
            assert_eq!(a, b, "{} signature mutation {index}", case.name);
        }
        for end in (0..case.signature.len()).step_by(11) {
            assert!(
                verification::verify_canister_sig(
                    case.message,
                    &case.signature[..end],
                    &case.public,
                    &case.root
                )
                .is_err()
            );
        }
        assert!(
            verification::verify_canister_sig(
                b"changed",
                &case.signature,
                &case.public,
                &case.root
            )
            .is_err()
        );
        assert!(
            ic_signature_verification::verify_canister_sig(
                b"changed",
                &case.signature,
                &case.public,
                &case.root
            )
            .is_err()
        );
    }
}

fn timed(iterations: usize, operation: &mut impl FnMut()) -> f64 {
    let started = Instant::now();
    for _ in 0..iterations {
        operation();
    }
    started.elapsed().as_nanos() as f64 / iterations as f64
}
fn measure(name: &str, bytes: usize, iterations: usize, mut a: impl FnMut(), mut b: impl FnMut()) {
    for _ in 0..16 {
        a();
        b();
    }
    let (mut x, mut y) = (Vec::new(), Vec::new());
    for round in 0..9 {
        if round % 2 == 0 {
            x.push(timed(iterations, &mut a));
            y.push(timed(iterations, &mut b));
        } else {
            y.push(timed(iterations, &mut b));
            x.push(timed(iterations, &mut a));
        }
    }
    x.sort_by(f64::total_cmp);
    y.sort_by(f64::total_cmp);
    println!(
        "{name}\t{bytes}\t{iterations}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{:.3}",
        x[0],
        x[4],
        x[8],
        y[0],
        y[4],
        y[8],
        x[4] / y[4]
    );
}

#[test]
#[ignore = "explicit release-mode native performance comparison"]
fn benchmark_signature_protocol() {
    assert!(!black_box(cfg!(debug_assertions)), "run with --release");
    println!(
        "workload\tinput_bytes\titerations_per_sample\towned_min_ns\towned_median_ns\towned_max_ns\treference_min_ns\treference_median_ns\treference_max_ns\towned_over_reference"
    );
    let principal = [0x5b; 29];
    let a = owned::CanisterSigPublicKey::new(Principal::from_slice(&principal), vec![0x39; 32]);
    let b = reference::CanisterSigPublicKey::new(
        principal.as_slice().try_into().unwrap(),
        a.seed.clone(),
    );
    let (raw, der) = (a.to_raw(), a.to_der());
    measure(
        "key-raw-encode",
        raw.len(),
        50_000,
        || {
            black_box(black_box(&a).to_raw());
        },
        || {
            black_box(black_box(&b).to_raw());
        },
    );
    measure(
        "key-der-encode",
        raw.len(),
        50_000,
        || {
            black_box(black_box(&a).to_der());
        },
        || {
            black_box(black_box(&b).to_der());
        },
    );
    measure(
        "key-raw-decode",
        raw.len(),
        50_000,
        || {
            black_box(owned::CanisterSigPublicKey::try_from_raw(black_box(&raw)).unwrap());
        },
        || {
            black_box(reference::CanisterSigPublicKey::try_from_raw(black_box(&raw)).unwrap());
        },
    );
    measure(
        "key-der-decode",
        der.len(),
        50_000,
        || {
            black_box(owned::CanisterSigPublicKey::try_from(black_box(der.as_slice())).unwrap());
        },
        || {
            black_box(
                reference::CanisterSigPublicKey::try_from(black_box(der.as_slice())).unwrap(),
            );
        },
    );
    for len in [32, 4096] {
        let payload = vec![0xc9; len];
        measure(
            &format!("sha256-{len}"),
            len,
            10_000,
            || {
                black_box(owned::hash_bytes(black_box(&payload)));
            },
            || {
                black_box(reference::hash_bytes(black_box(&payload)));
            },
        );
    }
    for case in cases() {
        measure(
            case.name,
            case.signature.len(),
            100,
            || {
                black_box(verification::verify_canister_sig(
                    black_box(case.message),
                    black_box(&case.signature),
                    black_box(&case.public),
                    black_box(&case.root),
                ))
                .unwrap();
            },
            || {
                black_box(ic_signature_verification::verify_canister_sig(
                    black_box(case.message),
                    black_box(&case.signature),
                    black_box(&case.public),
                    black_box(&case.root),
                ))
                .unwrap();
            },
        );
    }
}
