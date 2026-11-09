//! Independent upstream byte checks for the deliberately small encoding subset.
use candid as contract;
use ic_platform::candid_args;

#[test]
fn empty_and_unit_arguments_preserve_candid_arity() {
    assert_eq!(
        candid_args::empty().as_slice(),
        contract::encode_args(()).unwrap()
    );
    assert_eq!(candid_args::units(0), contract::encode_args(()).unwrap());
    assert_eq!(candid_args::units(1), contract::encode_args(((),)).unwrap());
    assert_eq!(
        candid_args::units(2),
        contract::encode_args(((), ())).unwrap()
    );
    assert_ne!(
        candid_args::units(1).as_slice(),
        candid_args::empty().as_slice()
    );
}

#[test]
fn text_preserves_unicode_and_leb_length_boundaries() {
    for text in ["", "hello", "héllo 🌍", "\0\"\\\n"] {
        assert_eq!(
            candid_args::text(text),
            contract::encode_args((text,)).unwrap()
        );
    }
    for length in [127, 128, 16383, 16384] {
        let text = "x".repeat(length);
        assert_eq!(
            candid_args::text(&text),
            contract::encode_args((text,)).unwrap()
        );
    }
}

#[test]
fn unit_count_crosses_the_single_byte_header_boundary() {
    let bytes = candid_args::units(128);
    assert_eq!(&bytes[..7], b"DIDL\0\x80\x01");
    assert_eq!(&bytes[7..], &[0x7f; 128]);
    contract::decode_args_with_config::<()>(&bytes, &contract::DecoderConfig::new()).unwrap();
}
