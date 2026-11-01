use super::system_text;

#[test]
fn system_text_preserves_utf8_and_compact_failure_reasons() {
    for text in ["", "method_name", "拒否"] {
        assert_eq!(system_text(text.as_bytes().to_vec(), "invalid"), text);
    }
    for reason in [
        "reject message is not valid UTF-8",
        "msg_method_name is not valid UTF-8",
    ] {
        let error = std::panic::catch_unwind(|| system_text(vec![0xff], reason))
            .expect_err("invalid UTF-8 must still panic");
        let message = error
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| error.downcast_ref::<&str>().copied());
        assert_eq!(message, Some(reason));
    }
}
