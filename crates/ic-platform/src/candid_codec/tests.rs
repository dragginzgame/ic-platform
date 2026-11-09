use super::*;
use candid as reference;

fn compare<T>(value: &T)
where
    T: Encode
        + reference::CandidType
        + for<'a> serde::Deserialize<'a>
        + PartialEq
        + std::fmt::Debug,
{
    let encoded = encode_one(value, 1_000_000).unwrap();
    let (decoded,): (T,) =
        reference::decode_args_with_config(&encoded, &reference::DecoderConfig::new()).unwrap();
    assert_eq!(&decoded, value);
    assert_eq!(encode_one(value, encoded.len()), Ok(encoded.clone()));
    assert_eq!(encode_one(value, encoded.len() - 1), Err(Error::Limit));
}

#[test]
fn scalars_match_independent_bytes() {
    macro_rules! scalar {
        ($($value:expr),* $(,)?) => {$({
            let value = $value;
            compare(&value);
            assert_eq!(encode_one(&value, 4096).unwrap(), reference::encode_one(value).unwrap());
        })*};
    }
    scalar!(
        (),
        false,
        true,
        0u8,
        u8::MAX,
        u16::MAX,
        u32::MAX,
        u64::MAX,
        i8::MIN,
        i16::MIN,
        i32::MIN,
        i64::MIN,
        -1i64,
        0i64,
        i64::MAX,
        0.0f32,
        -0.0f64,
        f64::INFINITY,
        f32::NEG_INFINITY
    );
    for bits in [0x7fc00000u32, 0xffc12345, 1, u32::MAX] {
        let value = f32::from_bits(bits);
        assert_eq!(
            encode_one(&value, 4096).unwrap(),
            reference::encode_one(value).unwrap()
        );
    }
}

#[test]
fn text_and_length_boundaries() {
    for length in [0, 1, 127, 128, 16383, 16384] {
        let value = "é".repeat(length);
        compare(&value);
        assert_eq!(
            encode_one(value.as_str(), 1_000_000).unwrap(),
            reference::encode_one(&value).unwrap()
        );
    }
}

#[test]
fn shared_generic_container_machinery() {
    compare(&None::<u64>);
    compare(&Some(None::<String>));
    compare(&Some(Some("hello".to_owned())));
    compare(&Vec::<u64>::new());
    compare(&vec![0u64, 127, 128, u64::MAX]);
    compare(&(42u64, Some(vec!["a".to_owned(), "🌍".to_owned()])));
    compare(&vec![vec![true, false], vec![], vec![true]]);
}

#[test]
fn malformed_trusted_schemas_are_refused() {
    static BAD: Type = Type::Record(&[(1, &Type::Primitive(-1)), (0, &Type::Primitive(-1))]);
    let mut table = Vec::new();
    assert_eq!(register(&BAD, &mut table, 1024), Err(Error::Schema));
    assert_eq!(
        register(&Type::Primitive(-18), &mut Vec::new(), 1024),
        Err(Error::Schema)
    );
    assert_eq!(
        register(
            &Type::Service(&[("x", &Type::Primitive(-1))]),
            &mut Vec::new(),
            1024,
        ),
        Err(Error::Schema)
    );
}

#[test]
fn recursive_schema_registration_terminates() {
    static LIST: Type = Type::Variant(&[(0, &Type::Primitive(-1)), (1, &NODE)]);
    static NODE: Type = Type::Record(&[(0, &Type::Primitive(-8)), (1, &LIST)]);
    let mut table = Vec::new();
    register(&LIST, &mut table, 1024).unwrap();
    assert_eq!(table.len(), 2);
    let mut output = Writer {
        bytes: Vec::new(),
        limit: 100,
    };
    header(&[&LIST], &mut output).unwrap();
    // The empty branch is a complete finite value of the recursive schema.
    output.unsigned(0).unwrap();
    reference::decode_args_with_config::<()>(&output.bytes, &reference::DecoderConfig::new())
        .unwrap();
}

#[test]
fn deterministic_nested_corpus_is_readable_upstream() {
    let mut seed = 0x12345678u64;
    for size in [0, 1, 2, 31, 127, 128, 1024] {
        let mut rows = Vec::new();
        for index in 0..size {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            rows.push((
                seed,
                if index % 3 == 0 {
                    None
                } else {
                    Some(format!("row-{index}-🌍"))
                },
            ));
        }
        compare(&rows);
    }
}

#[test]
fn reference_values_and_function_service_schemas_are_accepted() {
    static FUNCTION: Type = Type::Function {
        args: &[&Type::Primitive(-15)],
        results: &[&Type::Primitive(-8)],
        annotations: &[1],
    };
    static SERVICE: Type = Type::Service(&[("read", &FUNCTION)]);
    for schema in [&Type::Primitive(-24), &FUNCTION, &SERVICE] {
        let mut output = Writer {
            bytes: Vec::new(),
            limit: 4096,
        };
        header(&[schema], &mut output).unwrap();
        if matches!(schema, Type::Function { .. }) {
            output.bytes(&[1]).unwrap();
        }
        // An ordinary principal reference to the management canister.
        output.bytes(&[1, 0]).unwrap();
        if matches!(schema, Type::Function { .. }) {
            "read".write(&mut output).unwrap();
        }
        reference::decode_args_with_config::<()>(&output.bytes, &reference::DecoderConfig::new())
            .unwrap();
    }
}

#[test]
fn trusted_schema_traversal_uses_output_bound_without_a_fixed_type_cap() {
    let mut schema: &'static Type = &Type::Primitive(-1);
    for _ in 0..1024 {
        schema = Box::leak(Box::new(Type::Opt(schema)));
    }
    let mut table = Vec::new();
    register(schema, &mut table, 8192).unwrap();
    assert_eq!(table.len(), 1024);
    let aliases = super::schema::compact(&mut table).unwrap();
    assert_eq!(table.len(), 1024);
    assert_eq!(aliases.len(), 1024);
    assert_eq!(register(schema, &mut Vec::new(), 16), Err(Error::Limit));
}
