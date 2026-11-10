use super::*;

#[test]
fn key_encodings_preserve_issuer_and_seed() {
    for issuer in [vec![], vec![4], vec![0xab; 29]] {
        for seed in [vec![], vec![0, 1, 255], vec![0x42; 256]] {
            let key = CanisterSigPublicKey::new(Principal::from_slice(&issuer), seed.clone());
            let mut expected = vec![issuer.len() as u8];
            expected.extend_from_slice(&issuer);
            expected.extend_from_slice(&seed);
            assert_eq!(key.to_raw(), expected);
            assert_eq!(
                CanisterSigPublicKey::try_from_raw(&expected),
                Ok(key.clone())
            );
            let der = key.to_der();
            assert_eq!(&der[19..], expected);
            if der.len() < 130 {
                assert_eq!(CanisterSigPublicKey::try_from(der.as_slice()), Ok(key));
            } else {
                assert!(CanisterSigPublicKey::try_from(der.as_slice()).is_err());
            }
        }
    }
}

#[test]
fn raw_admission_rejects_missing_or_oversized_issuer() {
    assert!(CanisterSigPublicKey::try_from_raw(&[]).is_err());
    assert!(CanisterSigPublicKey::try_from_raw(&[2, 1]).is_err());
    let mut oversized = vec![30];
    oversized.extend_from_slice(&[0; 30]);
    assert!(CanisterSigPublicKey::try_from_raw(&oversized).is_err());
}

#[test]
fn der_admission_checks_every_envelope_field() {
    let valid = CanisterSigPublicKey::new(Principal::anonymous(), vec![9]).to_der();
    for index in [0, 1, 2, 3, 4, 15, 16, 17, 18] {
        let mut malformed = valid.clone();
        malformed[index] ^= 1;
        assert!(CanisterSigPublicKey::try_from(malformed.as_slice()).is_err());
    }
    for end in 0..valid.len() {
        assert!(CanisterSigPublicKey::try_from(&valid[..end]).is_err());
    }
    let mut trailing = valid;
    trailing.push(0);
    assert!(CanisterSigPublicKey::try_from(trailing.as_slice()).is_err());
}

#[test]
fn short_form_limit_does_not_change_historical_encoder() {
    for seed_len in [108, 109, 110, 127, 255, 256] {
        let key = CanisterSigPublicKey::new(Principal::management_canister(), vec![3; seed_len]);
        let der = key.to_der();
        assert_eq!(der[1], (18 + seed_len) as u8);
        assert_eq!(der[17], (2 + seed_len) as u8);
        assert_eq!(
            CanisterSigPublicKey::try_from(der.as_slice()).is_ok(),
            seed_len <= 109
        );
    }
}

#[test]
fn protocol_hashes_use_external_sha256_with_exact_framing() {
    for domain in [vec![], b"ic-state-root".to_vec(), vec![42; 256]] {
        let payload = b"opaque protocol bytes";
        let mut framed = vec![domain.len() as u8];
        framed.extend_from_slice(&domain);
        framed.extend_from_slice(payload);
        assert_eq!(hash_with_domain(&domain, payload), hash_bytes(framed));
    }
}

#[test]
fn root_key_admission_is_exact() {
    let key = extract_raw_root_pk_from_der(IC_ROOT_PK_DER).unwrap();
    assert_eq!(key.len(), 96);
    assert_eq!(key[0], 0x81);
    assert_eq!(key[95], 0xae);
    assert!(extract_raw_root_pk_from_der(&IC_ROOT_PK_DER[1..]).is_err());
    let mut changed = *IC_ROOT_PK_DER;
    changed[0] ^= 1;
    assert!(extract_raw_root_pk_from_der(&changed).is_err());
}
