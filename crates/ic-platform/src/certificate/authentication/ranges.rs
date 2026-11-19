//! Stream the selected certified range list without allocating a list of principals.
//! Adapted from ic-platform c24e3a59583eabcf5b6bb87987269c13a209a51f.
//! Toko Studios Network Licence 1.0; provenance in the certificate qualification record.

use super::{Principal, VerifyError as Error};
use serde::{
    Deserialize, Deserializer,
    de::{self, DeserializeSeed, SeqAccess, Visitor},
};
use std::{cell::Cell, fmt};

pub(super) fn authorize(
    bytes: &[u8],
    canister: Principal,
    maximum: usize,
    shard: Option<&[u8]>,
) -> Result<(), Error> {
    let exceeded = Cell::new(false);
    let mut decoder = serde_cbor::Deserializer::from_slice(bytes);
    let matched = Ranges {
        canister,
        maximum,
        shard,
        exceeded: &exceeded,
    }
    .deserialize(&mut decoder)
    .map_err(|_| {
        if exceeded.get() {
            Error::RangeLimit
        } else {
            Error::InvalidRanges
        }
    })?;
    decoder.end().map_err(|_| Error::InvalidRanges)?;
    if matched {
        Ok(())
    } else {
        Err(Error::NotAuthorized)
    }
}

struct Ranges<'a> {
    canister: Principal,
    maximum: usize,
    shard: Option<&'a [u8]>,
    exceeded: &'a Cell<bool>,
}
impl<'de> DeserializeSeed<'de> for Ranges<'_> {
    type Value = bool;
    fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<bool, D::Error> {
        d.deserialize_seq(self)
    }
}
impl<'de> Visitor<'de> for Ranges<'_> {
    type Value = bool;
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("bounded canister-range pairs")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<bool, A::Error> {
        if seq.size_hint().is_some_and(|n| n > self.maximum) {
            self.exceeded.set(true);
            return Err(de::Error::custom("canister-range limit"));
        }
        let mut count = 0;
        let mut matched = false;
        let mut previous: Option<Principal> = None;
        while let Some((PrincipalBytes(start), PrincipalBytes(end))) =
            seq.next_element_seed(Pair {
                allowed: count < self.maximum,
                exceeded: self.exceeded,
            })?
        {
            if start.as_slice() > end.as_slice()
                || previous.is_some_and(|old| old.as_slice() >= start.as_slice())
                || (count == 0 && self.shard.is_some_and(|label| label != start.as_slice()))
            {
                return Err(de::Error::custom("reversed canister range"));
            }
            matched |= start.as_slice() <= self.canister.as_slice()
                && self.canister.as_slice() <= end.as_slice();
            previous = Some(end);
            count += 1;
        }
        Ok(matched)
    }
}

struct Pair<'a> {
    allowed: bool,
    exceeded: &'a Cell<bool>,
}
impl<'de> DeserializeSeed<'de> for Pair<'_> {
    type Value = (PrincipalBytes, PrincipalBytes);
    fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
        if !self.allowed {
            self.exceeded.set(true);
            return Err(de::Error::custom("canister-range limit"));
        }
        // Tuple deserialization enforces exactly two elements, including indefinite arrays.
        <(PrincipalBytes, PrincipalBytes)>::deserialize(d)
    }
}

struct PrincipalBytes(Principal);
impl<'de> Deserialize<'de> for PrincipalBytes {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_any(PrincipalVisitor)
    }
}
struct PrincipalVisitor;
impl Visitor<'_> for PrincipalVisitor {
    type Value = PrincipalBytes;
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("principal byte string of at most 29 bytes")
    }

    fn visit_bytes<E: de::Error>(self, bytes: &[u8]) -> Result<Self::Value, E> {
        Principal::try_from_slice(bytes)
            .map(PrincipalBytes)
            .map_err(E::custom)
    }

    fn visit_byte_buf<E: de::Error>(self, bytes: Vec<u8>) -> Result<Self::Value, E> {
        self.visit_bytes(&bytes)
    }
}
