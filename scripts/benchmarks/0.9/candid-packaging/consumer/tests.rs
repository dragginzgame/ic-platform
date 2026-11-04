use super::*;

#[test]
fn registry_dtos_use_the_selected_codec_without_conversions() {
    let hex: String = include_str!("fixtures/management-snapshot-status.hex")
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    let bytes: Vec<_> = hex
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect();
    let statuses: Vec<CanisterStatusResult> = candid::decode_one(&bytes).unwrap();
    assert_eq!(statuses.len(), 3);
    for status in statuses {
        let wire = candid::encode_one(&status).unwrap();
        assert_eq!(encode(&status).unwrap(), wire);
        assert_eq!(decode(&wire).unwrap(), status);
    }
}

#[test]
fn registry_error_identity_is_retained() {
    let error: candid::Error = IDLDeserialize::new(b"invalid").err().unwrap();
    assert!(!error.to_string().is_empty());
}
