use super::*;
#[test]
fn explicit_adapter_reuses_the_runtime_and_preserves_guard_before_copy() {
    let generated = expand(
        true,
        quote!(
            limits = "LIMITS",
            empty_args = "reject",
            guard = "auth",
            platform = "::renamed"
        ),
        quote!(
            fn external(value: u64) -> u64 {
                value
            }
        ),
    )
    .unwrap()
    .to_string();
    assert!(generated.contains("renamed :: candid :: endpoint :: __candid_method"));
    assert!(generated.contains("renamed :: cdk :: futures :: internals"));
    let auth = generated.find("= auth ()").unwrap();
    let byte_guard = generated[auth..]
        .find("= __platform_candid_admit_external ()")
        .unwrap()
        + auth;
    let copy = generated
        .find("renamed :: cdk :: api :: msg_arg_data")
        .unwrap();
    assert!(auth < byte_guard && byte_guard < copy);
    assert!(!generated.contains(":: candid :: de"));
    assert!(!generated.contains("ic_cdk"));
}
#[test]
fn external_options_require_policy_and_refuse_implicit_codec_overrides() {
    for (attrs, text) in [
        (quote!(), "requires limits"),
        (quote!(limits = "LIMITS"), "requires empty_args"),
        (
            quote!(limits = "LIMITS", empty_args = "guess"),
            "reject or unit",
        ),
        (
            quote!(limits = "A", limits = "B", empty_args = "unit"),
            "duplicate",
        ),
        (
            quote!(
                limits = "LIMITS",
                empty_args = "unit",
                decode_with = "other"
            ),
            "unsupported",
        ),
    ] {
        assert!(
            expand(
                true,
                attrs,
                quote!(
                    fn external() {}
                )
            )
            .unwrap_err()
            .to_string()
            .contains(text)
        );
    }
}
