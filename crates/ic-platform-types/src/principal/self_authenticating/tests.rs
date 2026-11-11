use super::Principal;

#[test]
fn sha224_framing_matches_an_independent_digest() {
    // SHA-224("abc"), independently checked with OpenSSL, plus the IC suffix.
    let expected = [
        0x23, 0x09, 0x7d, 0x22, 0x34, 0x05, 0xd8, 0x22, 0x86, 0x42, 0xa4, 0x77, 0xbd, 0xa2, 0x55,
        0xb3, 0x2a, 0xad, 0xbc, 0xe4, 0xbd, 0xa0, 0xb3, 0xf7, 0xe3, 0x6c, 0x9d, 0xa7, 0x02,
    ];
    let bytes = b"abc".to_vec();
    for principal in [
        Principal::self_authenticating(b"abc"),
        Principal::self_authenticating(bytes.as_slice()),
        Principal::self_authenticating(&bytes),
        Principal::self_authenticating(bytes),
    ] {
        assert_eq!(principal.as_slice(), expected);
        assert_eq!(principal, Principal::from_slice(&expected));
    }
}
