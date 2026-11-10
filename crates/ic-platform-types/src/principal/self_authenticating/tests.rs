use super::Principal;

#[test]
fn delegates_without_key_admission_or_byte_changes() {
    for length in [0usize, 1, 28, 32, 44, 65, 91, 512, 4096] {
        let bytes: Vec<u8> = (0..length)
            .map(|index| (index as u8).wrapping_mul(19))
            .collect();
        let expected = ic_principal::Principal::self_authenticating(&bytes);
        let actual = Principal::self_authenticating(&bytes);
        assert_eq!(actual.as_slice(), expected.as_slice());
        assert_eq!(actual.len(), 29);
        assert_eq!(actual.to_text(), expected.to_text());
    }
}
