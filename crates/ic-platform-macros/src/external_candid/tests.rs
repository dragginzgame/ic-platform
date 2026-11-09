use super::*;
#[test]
fn explicit_adapter_reuses_the_runtime_and_preserves_guard_before_copy() {
    let generated = expand_owned(
        true,
        quote!(
            limits = "LIMITS",
            reply_bytes = "REPLY",
            arity = "exact",
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
    assert!(!generated.contains("__candid_method"));
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
            expand_owned(
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

#[test]
fn owned_adapter_requires_explicit_budgets_and_uses_the_same_guard_order() {
    let generated = expand_owned(
        true,
        quote!(
            limits = "LIMITS",
            reply_bytes = "REPLY",
            arity = "exact",
            empty_args = "reject",
            guard = "auth",
            platform = "::renamed",
            composite
        ),
        quote!(
            async fn external(value: u64) -> (u64, bool) {
                (value, true)
            }
        ),
    )
    .unwrap()
    .to_string();
    assert!(generated.contains("renamed :: candid_codec :: endpoint :: decode"));
    assert!(generated.contains("renamed :: candid_codec :: arguments :: encode_args"));
    assert!(generated.contains("renamed :: cdk :: futures :: internals"));
    assert!(!generated.contains("renamed :: candid ::"));
    assert!(!generated.contains("__candid_method"));
    let guard = generated.find("= auth ()").unwrap();
    let admit = generated[guard..]
        .find("= __platform_candid_admit_external ()")
        .unwrap()
        + guard;
    let copy = generated
        .find("renamed :: cdk :: api :: msg_arg_data")
        .unwrap();
    assert!(guard < admit && admit < copy);
    for attrs in [
        quote!(limits = "LIMITS", empty_args = "reject"),
        quote!(
            limits = "LIMITS",
            reply_bytes = "REPLY",
            empty_args = "unit"
        ),
        quote!(
            limits = "LIMITS",
            reply_bytes = "REPLY",
            arity = "guess",
            empty_args = "unit"
        ),
    ] {
        assert!(
            expand_owned(
                true,
                attrs,
                quote!(
                    fn external() {}
                )
            )
            .is_err()
        );
    }
}
