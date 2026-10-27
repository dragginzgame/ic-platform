//! Standalone host consumer: no direct Candid or parser dependency.
#![cfg(not(target_family = "wasm"))]

#[cfg(test)]
use ic_platform::candid_parser::{
    self as parser, CandidSource, Dec, FuncMode, IDLArgs, IDLProg, TypeContainer, TypeEnv,
    TypeInner,
};

#[cfg(test)]
const SERVICE: &str = "type Amount = nat64; service : (text) -> { read : () -> (Amount) query; write : (Amount) -> (); }";

#[test]
fn checked_external_services_preserve_constructor_types_and_argument_wire_bytes() {
    let program: IDLProg = SERVICE.parse().unwrap();
    assert!(
        !program
            .decs
            .iter()
            .any(|dec| matches!(dec, Dec::ImportType(_) | Dec::ImportServ(_)))
    );
    let mut env = TypeEnv::new();
    let actor = parser::check_prog(&mut env, &program).unwrap();
    let method = env.get_method(actor.as_ref().unwrap(), "read").unwrap();
    assert_eq!(method.modes, [FuncMode::Query]);
    assert!(matches!(
        env.trace_type(&method.rets[0]).unwrap().as_ref(),
        TypeInner::Nat64
    ));

    let (init, (env, service)) = parser::instantiate_candid(CandidSource::Text(SERVICE)).unwrap();
    assert!(matches!(init[0].as_ref(), TypeInner::Text));
    let method = env.get_method(&service, "write").unwrap();
    let args = parser::parse_idl_args("(42)").unwrap();
    let bytes = args.to_bytes_with_types(&env, &method.args).unwrap();
    // One nat64 argument, independently specified little-endian Candid bytes.
    assert_eq!(bytes, b"DIDL\x00\x01\x78\x2a\x00\x00\x00\x00\x00\x00\x00");
    assert_eq!(
        IDLArgs::from_bytes_with_types(&bytes, &env, &method.args)
            .unwrap()
            .to_string(),
        "(42 : nat64)"
    );
    assert!(
        parser::parse_idl_args("(\"wrong type\")")
            .unwrap()
            .to_bytes_with_types(&env, &method.args)
            .is_err()
    );

    let mut rust = TypeContainer::new();
    let expected = rust.add::<u64>();
    let mut env = env.clone();
    let expected = env.merge_type(rust.env, expected);
    parser::type_equal(
        &mut std::collections::HashSet::new(),
        &env,
        &method.args[0],
        &expected,
    )
    .unwrap();
}

#[test]
fn malformed_input_and_service_drift_preserve_typed_errors() {
    let _: parser::Error = "service : {".parse::<IDLProg>().unwrap_err();
    let mut env = TypeEnv::new();
    let undefined: IDLProg = "service : { read : () -> (Missing) query }"
        .parse()
        .unwrap();
    let _: parser::Error = parser::check_prog(&mut env, &undefined).unwrap_err();
    let imported: IDLProg = "import \"not-loaded.did\"; service : {}".parse().unwrap();
    assert!(matches!(&imported.decs[0], Dec::ImportType(_)));
    let equal: parser::Result<()> =
        parser::service_equal(CandidSource::Text(SERVICE), CandidSource::Text(SERVICE));
    equal.unwrap();
    assert!(
        parser::service_equal(
            CandidSource::Text("service : { read : () -> (nat64) query }"),
            CandidSource::Text("service : { read : () -> (text) query }")
        )
        .is_err()
    );
}
