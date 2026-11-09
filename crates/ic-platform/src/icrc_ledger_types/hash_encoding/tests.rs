use super::*;
#[test]
fn preserved_encoder_matches_supplier_for_wide_values_and_writer_errors() {
    for bits in [0usize, 7, 63, 64, 65, 127, 128, 256, 8192] {
        let boundary = num_bigint::BigUint::from(1u8) << bits;
        for value in [&boundary - 1u8, boundary.clone(), &boundary + 1u8] {
            let own = Nat(value.clone());
            let external = value.to_string().parse::<candid::Nat>().unwrap();
            let mut actual = Vec::new();
            let mut expected = Vec::new();
            own.encode(&mut actual).unwrap();
            external.encode(&mut expected).unwrap();
            assert_eq!(actual, expected);
            let mut own_short = [0u8; 1];
            let mut ref_short = [0u8; 1];
            assert_eq!(
                own.encode(&mut own_short.as_mut_slice()).is_ok(),
                external.encode(&mut ref_short.as_mut_slice()).is_ok()
            );
            assert_eq!(own_short, ref_short);
        }
    }
}
