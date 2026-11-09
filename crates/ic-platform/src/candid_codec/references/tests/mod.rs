use super::*;
use crate::candid_codec::{
    decoder::{Decode, Limits, decode_one},
    encode_one,
};
use std::marker::PhantomData;

trait Signature {
    const TYPE: &'static Type;
    type Reference: candid::CandidType
        + for<'a> serde::Deserialize<'a>
        + From<candid::types::reference::Func>;
}
struct Typed<S>(Func, PhantomData<S>);
impl<S: Signature> Encode for Typed<S> {
    const TYPE: &'static Type = S::TYPE;
    fn write(&self, writer: &mut Writer) -> Result<(), Error> {
        self.0.write(writer)
    }
}
impl<'a, S: Signature> Decode<'a> for Typed<S> {
    fn read(reader: &mut Reader<'a>, wire: i64) -> Result<Self, decoder::Error> {
        Func::read(reader, wire, S::TYPE).map(|v| Self(v, PhantomData))
    }
}
macro_rules! signature {
    ($name:ident, $args:expr, $results:expr, $modes:expr; $($upstream:tt)+) => {
        mod $name {
            use super::*;
            candid::define_function!(pub(super) Reference : $($upstream)+);
            pub(super) struct Schema;
            impl Signature for Schema { const TYPE:&'static Type=&Type::Function{args:$args,results:$results,annotations:$modes}; type Reference=Reference; }
        }
    };
}
const NAT: &Type = &Type::Primitive(-3);
const INT: &Type = &Type::Primitive(-4);
const U64: &Type = &Type::Primitive(-8);
const TEXT: &Type = &Type::Primitive(-15);
signature!(nat, &[NAT], &[NAT], &[1]; (candid::Nat)->(candid::Nat) query);
signature!(int, &[INT], &[INT], &[1]; (candid::Int)->(candid::Int) query);
signature!(wide, &[INT], &[NAT], &[1]; (candid::Int)->(candid::Nat) query);
signature!(narrow, &[NAT], &[INT], &[1]; (candid::Nat)->(candid::Int) query);
signature!(update, &[NAT], &[NAT], &[]; (candid::Nat)->(candid::Nat));
signature!(composite, &[NAT], &[NAT], &[3]; (candid::Nat)->(candid::Nat) composite_query);
signature!(oneway, &[NAT], &[], &[2]; (candid::Nat)->() oneway);
signature!(none, &[], &[], &[1]; ()->() query);
signature!(optional, &[&Type::Opt(NAT)], &[&Type::Opt(TEXT)], &[1]; (Option<candid::Nat>)->(Option<String>) query);
signature!(more, &[NAT,U64], &[NAT,U64], &[1]; (candid::Nat,u64)->(candid::Nat,u64) query);
signature!(vector, &[&Type::Vec(INT)], &[&Type::Vec(NAT)], &[1]; (Vec<candid::Int>)->(Vec<candid::Nat>) query);
#[derive(candid::CandidType, serde::Deserialize)]
struct Small {
    a: u64,
}
#[derive(candid::CandidType, serde::Deserialize)]
struct Large {
    a: u64,
    b: Option<String>,
}
const SMALL: &Type = &Type::Record(&[(97, U64)]);
const LARGE: &Type = &Type::Record(&[(97, U64), (98, &Type::Opt(TEXT))]);
signature!(small, &[SMALL], &[SMALL], &[1]; (Small)->(Small) query);
signature!(large, &[LARGE], &[LARGE], &[1]; (Large)->(Large) query);
#[derive(candid::CandidType, serde::Deserialize)]
enum Narrow {
    A(u64),
}
#[derive(candid::CandidType, serde::Deserialize)]
enum Wide {
    A(u64),
    B(String),
}
signature!(vnarrow, &[], &[&Type::Variant(&[(65,U64)])], &[1]; ()->(Narrow) query);
signature!(vwide, &[], &[&Type::Variant(&[(65,U64),(66,TEXT)])], &[1]; ()->(Wide) query);
#[derive(candid::CandidType, serde::Deserialize)]
struct Node(u64, Option<Box<Node>>);
static NODE: Type = Type::Record(&[(0, U64), (1, &NODE_OPT)]);
static NODE_OPT: Type = Type::Opt(&NODE);
signature!(recursive, &[&NODE], &[&NODE], &[1]; (Node)->(Node) query);
fn limits() -> Limits {
    Limits {
        bytes: 65536,
        header: 32768,
        types: 1024,
        work: 1_000_000,
        allocation: 1_000_000,
    }
}
fn encoded_reference<S: Signature>() -> Vec<u8> {
    candid::encode_one(S::Reference::from(candid::types::reference::Func {
        principal: candid::Principal::from_slice(&[42; 29]),
        method: "archive_🌍".into(),
    }))
    .unwrap()
}
fn compare<S: Signature, T: Signature>() {
    let bytes = encoded_reference::<S>();
    let upstream =
        candid::decode_args_with_config::<(T::Reference,)>(&bytes, &candid::DecoderConfig::new())
            .is_ok();
    let ours = decode_one::<Typed<T>>(&bytes, limits());
    assert_eq!(
        ours.is_ok(),
        upstream,
        "{} -> {}: {:?}",
        std::any::type_name::<S>(),
        std::any::type_name::<T>(),
        ours.as_ref().err()
    );
    if let Ok(value) = ours {
        assert_eq!(value.0.principal, Principal::from_slice(&[42; 29]));
        assert_eq!(value.0.method, "archive_🌍");
        let encoded = encode_one(&value, 65536).unwrap();
        assert!(
            candid::decode_args_with_config::<(T::Reference,)>(
                &encoded,
                &candid::DecoderConfig::new()
            )
            .is_ok()
        );
    }
}
macro_rules! targets {
    ($source:ty) => {
        compare::<$source, nat::Schema>();
        compare::<$source, int::Schema>();
        compare::<$source, wide::Schema>();
        compare::<$source, narrow::Schema>();
        compare::<$source, update::Schema>();
        compare::<$source, composite::Schema>();
        compare::<$source, oneway::Schema>();
        compare::<$source, none::Schema>();
        compare::<$source, optional::Schema>();
        compare::<$source, more::Schema>();
        compare::<$source, vector::Schema>();
        compare::<$source, small::Schema>();
        compare::<$source, large::Schema>();
        compare::<$source, vnarrow::Schema>();
        compare::<$source, vwide::Schema>();
        compare::<$source, recursive::Schema>();
    };
}
#[test]
fn signature_variance_modes_width_optional_and_recursive_parity() {
    targets!(nat::Schema);
    targets!(int::Schema);
    targets!(wide::Schema);
    targets!(narrow::Schema);
    targets!(update::Schema);
    targets!(composite::Schema);
    targets!(oneway::Schema);
    targets!(none::Schema);
    targets!(optional::Schema);
    targets!(more::Schema);
    targets!(vector::Schema);
    targets!(small::Schema);
    targets!(large::Schema);
    targets!(vnarrow::Schema);
    targets!(vwide::Schema);
    targets!(recursive::Schema);
}
#[test]
fn reference_payloads_and_budgets_fail_closed() {
    let bytes = encoded_reference::<recursive::Schema>();
    for n in 0..bytes.len() {
        assert!(decode_one::<Typed<recursive::Schema>>(&bytes[..n], limits()).is_err());
    }
    let mut extra = bytes.clone();
    extra.push(0);
    assert!(decode_one::<Typed<recursive::Schema>>(&extra, limits()).is_err());
    for budget in [0, 1, 16] {
        assert!(matches!(
            decode_one::<Typed<recursive::Schema>>(
                &bytes,
                Limits {
                    allocation: budget,
                    ..limits()
                }
            ),
            Err(decoder::Error::Limit)
        ));
        assert!(matches!(
            decode_one::<Typed<recursive::Schema>>(
                &bytes,
                Limits {
                    work: budget,
                    ..limits()
                }
            ),
            Err(decoder::Error::Limit)
        ));
    }
    let bytes = encoded_reference::<update::Schema>();
    assert!(decode_one::<Typed<nat::Schema>>(&bytes, limits()).is_err());
    assert!(
        decode_one::<Option<Typed<nat::Schema>>>(&bytes, limits())
            .unwrap()
            .is_none()
    );
}

candid::define_service!(SmallService : {"go": candid::func!(() -> () query)});
candid::define_service!(LargeService : {"go": candid::func!(() -> () query); "ping": candid::func!(() -> () query)});
const EMPTY_FN: &Type = &Type::Function {
    args: &[],
    results: &[],
    annotations: &[1],
};
const SMALL_SERVICE: &Type = &Type::Service(&[("go", EMPTY_FN)]);
const LARGE_SERVICE: &Type = &Type::Service(&[("go", EMPTY_FN), ("ping", EMPTY_FN)]);
signature!(small_service, &[SMALL_SERVICE], &[], &[1]; (SmallService)->() query);
signature!(large_service, &[LARGE_SERVICE], &[], &[1]; (LargeService)->() query);
signature!(principal, &[&Type::Primitive(-24)], &[], &[1]; (candid::Principal)->() query);
#[test]
fn service_width_and_service_principal_subtyping() {
    compare::<small_service::Schema, large_service::Schema>();
    compare::<large_service::Schema, small_service::Schema>();
    compare::<small_service::Schema, principal::Schema>();
    compare::<principal::Schema, small_service::Schema>();
    compare::<large_service::Schema, large_service::Schema>();
}
#[test]
fn malformed_reference_tags_lengths_and_utf8_refuse() {
    let bytes = encoded_reference::<nat::Schema>();
    let principal = bytes
        .windows(29)
        .position(|bytes| bytes == [42; 29])
        .unwrap();
    for offset in [principal - 3, principal - 2] {
        let mut bad = bytes.clone();
        bad[offset] = 0;
        assert!(matches!(
            decode_one::<Typed<nat::Schema>>(&bad, limits()),
            Err(decoder::Error::Unsupported)
        ));
    }
    let mut bad = bytes.clone();
    bad[principal - 1] = 30;
    assert!(matches!(
        decode_one::<Typed<nat::Schema>>(&bad, limits()),
        Err(decoder::Error::Malformed)
    ));
    let mut bad = bytes.clone();
    *bad.last_mut().unwrap() = 255;
    assert!(matches!(
        decode_one::<Typed<nat::Schema>>(&bad, limits()),
        Err(decoder::Error::Malformed)
    ));
    for index in 0..bytes.len() {
        for byte in [0, 1, 127, 128, 255] {
            let mut mutation = bytes.clone();
            mutation[index] = byte;
            if let Ok(owned) = decode_one::<Typed<nat::Schema>>(&mutation, limits()) {
                let external = candid::decode_args_with_config::<(nat::Reference,)>(
                    &mutation,
                    &candid::DecoderConfig::new(),
                )
                .unwrap()
                .0;
                assert_eq!(
                    owned.0.principal.as_slice(),
                    external.0.principal.as_slice()
                );
                assert_eq!(owned.0.method, external.0.method);
            }
        }
    }
}
