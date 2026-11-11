//! Project the owned schema into the independent oracle's type representation.
//! Frozen independent binary headers supply the expected protocol schemas.
use crate::candid_codec::{Encode, Type};
use candid::types::{Field, Function, Label, Type as ExternalType, TypeEnv, TypeInner};
use std::collections::HashMap;
use std::rc::Rc;

/// Decode trusted frozen schema evidence, never application input.
#[cfg(any(feature = "icrc-ledger-types", feature = "nns-registry-types"))]
pub(crate) fn frozen_schema(source: &str) -> TypeEnv {
    use binrw::BinRead;
    #[derive(serde::Deserialize)]
    struct Fixture {
        names: Vec<String>,
        hex: String,
    }
    let fixture: Fixture = serde_json::from_str(source).unwrap();
    let bytes = super::decode_hex(&fixture.hex);
    let mut cursor = std::io::Cursor::new(&bytes);
    let header = candid::binary_parser::Header::read_le_args(&mut cursor, (Some(1024),)).unwrap();
    let (mut env, types) = header.to_types().unwrap();
    assert_eq!(types.len(), fixture.names.len());
    // Each schema was captured as an absent optional value, including recursive
    // schemas and function references. The remaining bytes must be those Nones.
    assert_eq!(&bytes[cursor.position() as usize..], vec![0; types.len()]);
    for (name, ty) in fixture.names.into_iter().zip(types) {
        let resolved = env.trace_type(&ty).unwrap();
        let TypeInner::Opt(inner) = resolved.as_ref() else {
            panic!("expected optional schema envelope")
        };
        assert!(env.0.insert(name, inner.clone()).is_none());
    }
    env
}

/// Compares an owned schema against one named in an independent binary fixture.
#[cfg(feature = "icrc-ledger-types")]
pub(crate) fn assert_named_schema<T: Encode>(env: &TypeEnv, name: &str) {
    let mut env = env.clone();
    let declared = env.find_type(name).unwrap().clone();
    let (owned_env, actual) = schema::<T>();
    let actual = env.merge_type(owned_env, actual);
    candid::types::subtype::equal(
        &mut std::collections::HashSet::new(),
        &env,
        &declared,
        &actual,
    )
    .unwrap_or_else(|e| panic!("{name}: {e}"));
}

pub(crate) fn schema<T: Encode>() -> (TypeEnv, ExternalType) {
    let mut env = TypeEnv::new();
    let ty = project(T::TYPE, &mut env, &mut HashMap::new());
    // Use the upstream equality checker to merge equivalent declarations. Named
    // aliases must not manufacture duplicate wire definitions in the oracle.
    let mut names: Vec<_> = env.0.keys().cloned().collect();
    names.sort_by_key(|name| name[5..].parse::<usize>().unwrap());
    let mut aliases = std::collections::BTreeMap::new();
    for (index, name) in names.iter().enumerate() {
        for earlier in &names[..index] {
            if aliases.contains_key(earlier) {
                continue;
            }
            if candid::types::subtype::equal(
                &mut std::collections::HashSet::new(),
                &env,
                env.find_type(name).unwrap(),
                env.find_type(earlier).unwrap(),
            )
            .is_ok()
            {
                aliases.insert(name.clone(), earlier.clone());
                break;
            }
        }
    }
    env.0 = env
        .0
        .into_iter()
        .filter(|(name, _)| !aliases.contains_key(name))
        .map(|(name, ty)| (name, ty.subst(&aliases)))
        .collect();
    (env, ty.subst(&aliases))
}
fn project(
    ty: &'static Type,
    env: &mut TypeEnv,
    seen: &mut HashMap<usize, String>,
) -> ExternalType {
    let ty = ty.resolved().unwrap();
    if let Type::Primitive(code) = ty {
        return match code {
            -1 => TypeInner::Null,
            -2 => TypeInner::Bool,
            -3 => TypeInner::Nat,
            -4 => TypeInner::Int,
            -5 => TypeInner::Nat8,
            -6 => TypeInner::Nat16,
            -7 => TypeInner::Nat32,
            -8 => TypeInner::Nat64,
            -9 => TypeInner::Int8,
            -10 => TypeInner::Int16,
            -11 => TypeInner::Int32,
            -12 => TypeInner::Int64,
            -13 => TypeInner::Float32,
            -14 => TypeInner::Float64,
            -15 => TypeInner::Text,
            -16 => TypeInner::Reserved,
            -17 => TypeInner::Empty,
            -24 => TypeInner::Principal,
            _ => panic!("unknown primitive"),
        }
        .into();
    }
    let key = std::ptr::from_ref(ty) as usize;
    if let Some(name) = seen.get(&key) {
        return TypeInner::Var(name.clone()).into();
    }
    let name = format!("Owned{}", seen.len());
    seen.insert(key, name.clone());
    let value = match ty {
        Type::Opt(inner) => TypeInner::Opt(project(inner, env, seen)),
        Type::Vec(inner) => TypeInner::Vec(project(inner, env, seen)),
        Type::Record(fields) | Type::Variant(fields) => {
            let fields = fields
                .iter()
                .map(|(id, ty)| Field {
                    id: Rc::new(Label::Id(*id)),
                    ty: project(ty, env, seen),
                })
                .collect();
            if matches!(ty, Type::Record(_)) {
                TypeInner::Record(fields)
            } else {
                TypeInner::Variant(fields)
            }
        }
        Type::Function {
            args,
            results,
            annotations,
        } => TypeInner::Func(Function {
            args: args.iter().map(|ty| project(ty, env, seen)).collect(),
            rets: results.iter().map(|ty| project(ty, env, seen)).collect(),
            modes: annotations
                .iter()
                .map(|m| match m {
                    1 => candid::types::FuncMode::Query,
                    2 => candid::types::FuncMode::Oneway,
                    3 => candid::types::FuncMode::CompositeQuery,
                    _ => panic!("unknown mode"),
                })
                .collect(),
        }),
        Type::Service(methods) => TypeInner::Service(
            methods
                .iter()
                .map(|(name, ty)| ((*name).to_owned(), project(ty, env, seen)))
                .collect(),
        ),
        _ => unreachable!(),
    };
    env.0.insert(name.clone(), value.into());
    TypeInner::Var(name).into()
}
