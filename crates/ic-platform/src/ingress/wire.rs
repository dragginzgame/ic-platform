//! Private strict map/byte adapters; no generic public codec surface.
use super::SignedDelegation;
use candidate_types::Principal;
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, MapAccess, Visitor, value::MapAccessDeserializer},
};
use std::{fmt, marker::PhantomData};

pub(super) fn map<'de, D: Deserializer<'de>, T: Deserialize<'de>>(d: D) -> Result<T, D::Error> {
    struct Map<T>(PhantomData<T>);
    impl<'de, T: Deserialize<'de>> Visitor<'de> for Map<T> {
        type Value = T;
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("CBOR map")
        }

        fn visit_map<M: MapAccess<'de>>(self, m: M) -> Result<T, M::Error> {
            T::deserialize(MapAccessDeserializer::new(m))
        }
    }
    d.deserialize_map(Map(PhantomData))
}

pub(super) fn optional_map<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    d: D,
) -> Result<Option<T>, D::Error> {
    map(d).map(Some)
}

pub(super) fn present<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    d: D,
) -> Result<Option<T>, D::Error> {
    T::deserialize(d).map(Some)
}

pub(super) fn bytes<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
    struct Bytes;
    impl Visitor<'_> for Bytes {
        type Value = Vec<u8>;
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("CBOR byte string")
        }

        fn visit_bytes<E: de::Error>(self, bytes: &[u8]) -> Result<Self::Value, E> {
            Ok(bytes.to_vec())
        }

        fn visit_byte_buf<E: de::Error>(self, bytes: Vec<u8>) -> Result<Self::Value, E> {
            Ok(bytes)
        }
    }
    d.deserialize_any(Bytes)
}

pub(super) fn optional_bytes<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Vec<u8>>, D::Error> {
    bytes(d).map(Some)
}

pub(super) fn principal<'de, D: Deserializer<'de>>(d: D) -> Result<Principal, D::Error> {
    Principal::try_from_slice(&bytes(d)?).map_err(de::Error::custom)
}
#[derive(Deserialize)]
#[serde(transparent)]
struct Bytes(#[serde(deserialize_with = "bytes")] Vec<u8>);
#[derive(Deserialize)]
#[serde(transparent)]
struct Id(#[serde(deserialize_with = "principal")] Principal);
#[derive(Deserialize)]
#[serde(transparent)]
struct DelegationMap(#[serde(deserialize_with = "map")] SignedDelegation);
pub(super) fn paths<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Vec<Vec<u8>>>, D::Error> {
    Vec::<Vec<Bytes>>::deserialize(d).map(|paths| {
        paths
            .into_iter()
            .map(|p| p.into_iter().map(|b| b.0).collect())
            .collect()
    })
}

pub(super) fn targets<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Vec<Principal>>, D::Error> {
    Vec::<Id>::deserialize(d).map(|ids| Some(ids.into_iter().map(|id| id.0).collect()))
}

pub(super) fn delegations<'de, D: Deserializer<'de>>(
    d: D,
) -> Result<Option<Vec<SignedDelegation>>, D::Error> {
    Vec::<DelegationMap>::deserialize(d)
        .map(|entries| Some(entries.into_iter().map(|entry| entry.0).collect()))
}

pub(super) fn serialize_paths<S: Serializer>(
    paths: &[Vec<Vec<u8>>],
    s: S,
) -> Result<S::Ok, S::Error> {
    paths
        .iter()
        .map(|path| {
            path.iter()
                .map(|bytes| serde_bytes::Bytes::new(bytes))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
        .serialize(s)
}
