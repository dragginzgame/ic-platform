use super::*;
use candid_parser::utils::{CandidSource, service_equal};

#[test]
fn limits_modes_and_names_are_checked() {
    let methods = [Method::new::<(u64,), (bool,)>("a\"\\\nλ", Mode::Query)];
    let did = export(&methods, 4096).unwrap();
    assert!(service_equal(CandidSource::Text(&did), CandidSource::Text(&did)).is_ok());
    assert_eq!(export(&methods, did.len()).unwrap(), did);
    assert_eq!(export(&methods, did.len() - 1), Err(Error::Limit));
    assert_eq!(export(&[methods[0], methods[0]], 4096), Err(Error::Schema));
    assert_eq!(
        export(&[Method::new::<(), (u8,)>("bad", Mode::Oneway)], 4096),
        Err(Error::Schema)
    );
    for (mode, expected) in [
        (Mode::Update, ""),
        (Mode::Query, "query"),
        (Mode::Oneway, "oneway"),
        (Mode::CompositeQuery, "composite_query"),
    ] {
        let did = export(&[Method::new::<(), ()>("m", mode)], 4096).unwrap();
        let frozen = format!("service : {{ m : () -> () {expected}; }}");
        service_equal(CandidSource::Text(&did), CandidSource::Text(&frozen)).unwrap();
    }
}

#[test]
fn recursive_schemas_and_reference_signatures_preserve_equality() {
    fn node() -> &'static Type {
        &NODE
    }
    static NODE: Type = Type::Record(&[
        (0, &Type::Primitive(-3)),
        (1, &Type::Opt(&Type::Deferred(node))),
    ]);
    static CALLBACK: Type = Type::Function {
        args: &[&NODE],
        results: &[&Type::Primitive(-24)],
        annotations: &[1],
    };
    static SERVICE: Type = Type::Service(&[("callback", &CALLBACK)]);
    static CHOICE: Type = Type::Variant(&[
        (0, &Type::Vec(&Type::Primitive(-5))),
        (1, &Type::Primitive(-1)),
    ]);
    static ARGS: &[&Type] = &[&NODE, &CALLBACK, &SERVICE, &CHOICE];
    let method = Method {
        name: "walk",
        args: ARGS,
        results: &[],
        mode: Mode::Update,
    };
    let did = export(&[method], 4096).unwrap();
    let frozen = "type Node = record { nat; opt Node }; type Callback = func (Node) -> (principal) query; service : { walk : (Node, Callback, service { callback : Callback }, variant { 0 : blob; 1 : null }) -> () }";
    service_equal(CandidSource::Text(&did), CandidSource::Text(frozen)).unwrap();
    // Aliases and presentation order are not schema changes.
    assert_eq!(export(&[method], 4096).unwrap(), did);
}

#[test]
fn invalid_schema_is_rejected_by_the_binary_schema_owner() {
    static INVALID: &[&Type] = &[
        &Type::Primitive(-99),
        &Type::Record(&[(1, &Type::Primitive(-1)), (0, &Type::Primitive(-1))]),
        &Type::Function {
            args: &[],
            results: &[],
            annotations: &[9],
        },
        &Type::Service(&[("bad", &Type::Primitive(-1))]),
    ];
    for ty in INVALID {
        let method = Method {
            name: "bad",
            args: std::slice::from_ref(ty),
            results: &[],
            mode: Mode::Update,
        };
        assert_eq!(export(&[method], 4096), Err(Error::Schema));
    }
}
