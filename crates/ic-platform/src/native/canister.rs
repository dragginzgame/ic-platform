//! Certified canister facts; endpoint admission and lifecycle policy stay external.
use super::{Principal, ReadClient, ReadError, TimePolicy, authenticate_certificate};
use ic_certification::LookupResult;
use serde::de::{DeserializeSeed, Error as _, SeqAccess, Visitor};

/// Refusal of a certified canister observation; no partial evidence escapes.
#[derive(Debug, thiserror::Error)]
pub enum CanisterReadError {
    /// Existing transport, certificate, missing-path or malformed-packet refusal.
    #[error(transparent)]
    Read(#[from] ReadError),
    /// The selected controller path is pruned, not proven absent.
    #[error("certified controller path is pruned")]
    PrunedPath,
    /// The explicit controller count ceiling was exceeded.
    #[error("certified controller count exceeds allowance")]
    ControllerLimit,
}

/// Immutable controller and hosting-subnet observation from one certificate.
/// No claim about later routing, authorization or upgrade permission is made.
#[derive(Debug)]
pub struct CanisterObservation {
    canister: Principal,
    subnet: Principal,
    controllers: Vec<Principal>,
    certificate: Vec<u8>,
    time_ns: u64,
}
impl CanisterObservation {
    /// Exact canister whose certified controller leaf was admitted.
    #[must_use]
    pub const fn canister(&self) -> Principal {
        self.canister
    }
    /// Delegated signing subnet, or the explicitly trusted root-subnet binding.
    #[must_use]
    pub const fn subnet(&self) -> Principal {
        self.subnet
    }
    /// Certified controller identities in their source order.
    #[must_use]
    pub fn controllers(&self) -> &[Principal] {
        &self.controllers
    }
    /// Exact original certificate bytes, retained for independent replay.
    #[must_use]
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    /// Authenticated signing time under the supplied policy.
    #[must_use]
    pub const fn time_ns(&self) -> u64 {
        self.time_ns
    }
}
impl ReadClient {
    /// Read certified controllers and hosting subnet in one bounded observation.
    /// Delegated range authority is checked for this exact canister. A direct-root
    /// reply requires its canister-specific controller leaf and uses only the
    /// independently admitted `ReadConfig::root_subnet` binding. No routing
    /// identity is guessed from the canister ID, and no root discovery occurs.
    /// `maximum_controllers` bounds the result count before each element allocation;
    /// `message_bytes` bounds input, not total parser work or process memory.
    ///
    /// # Errors
    /// Refuses transport, certificate signature/range/time/resource failures,
    /// absent or pruned controller paths, malformed controller CBOR and count limits.
    pub async fn read_canister(
        &self,
        canister: Principal,
        expiry: u64,
        time: TimePolicy,
        maximum_controllers: usize,
    ) -> Result<CanisterObservation, CanisterReadError> {
        let bytes = self
            .state(
                "canister",
                canister,
                vec![
                    vec![
                        b"canister".to_vec(),
                        canister.as_slice().to_vec(),
                        b"controllers".to_vec(),
                    ],
                    vec![b"time".to_vec()],
                ],
                expiry,
            )
            .await?;
        self.admit_canister(bytes, canister, time, maximum_controllers)
    }
    pub(super) fn admit_canister(
        &self,
        bytes: Vec<u8>,
        canister: Principal,
        time: TimePolicy,
        maximum_controllers: usize,
    ) -> Result<CanisterObservation, CanisterReadError> {
        let evidence = authenticate_certificate(
            self.decode(&bytes)?,
            &self.config.root,
            canister,
            time,
            self.config.verification,
        )
        .map_err(ReadError::from)?;
        let leaf = match evidence.tree().lookup_path([
            b"canister".as_slice(),
            canister.as_slice(),
            b"controllers".as_slice(),
        ]) {
            LookupResult::Found(leaf) => leaf,
            LookupResult::Absent => return Err(ReadError::MissingPath.into()),
            LookupResult::Unknown => return Err(CanisterReadError::PrunedPath),
            LookupResult::Error => return Err(ReadError::Packet.into()),
        };
        let mut decoder = serde_cbor::Deserializer::from_slice(leaf);
        let mut limit = false;
        let controllers = Controllers {
            maximum: maximum_controllers,
            limit: &mut limit,
        }
        .deserialize(&mut decoder)
        .map_err(|_| {
            if limit {
                CanisterReadError::ControllerLimit
            } else {
                ReadError::Packet.into()
            }
        })?;
        decoder.end().map_err(|_| ReadError::Packet)?;
        let subnet = evidence
            .delegation()
            .map_or(self.config.root_subnet, |(id, _, _)| id);
        let time_ns = evidence.time_ns();
        Ok(CanisterObservation {
            canister,
            subnet,
            controllers,
            certificate: bytes,
            time_ns,
        })
    }
}
struct Controllers<'a> {
    maximum: usize,
    limit: &'a mut bool,
}
impl<'de> DeserializeSeed<'de> for Controllers<'_> {
    type Value = Vec<Principal>;
    fn deserialize<D: serde::Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
        d.deserialize_seq(self)
    }
}
impl<'de> Visitor<'de> for Controllers<'_> {
    type Value = Vec<Principal>;
    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("bounded array of principal byte strings")
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        if seq.size_hint().is_some_and(|n| n > self.maximum) {
            *self.limit = true;
            return Err(A::Error::custom("controller limit"));
        }
        let mut result = Vec::new();
        // Borrow each byte string. No wire-provided count or byte length is used
        // to allocate a controller buffer before validation.
        while let Some(Controller(p)) = seq.next_element::<Controller>()? {
            if result.len() == self.maximum {
                *self.limit = true;
                return Err(A::Error::custom("controller limit"));
            }
            if result.contains(&p) {
                return Err(A::Error::custom("duplicate controller"));
            }
            result.try_reserve(1).map_err(A::Error::custom)?;
            result.push(p);
        }
        Ok(result)
    }
}

struct Controller(Principal);
impl<'de> serde::Deserialize<'de> for Controller {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Bytes;
        impl<'de> Visitor<'de> for Bytes {
            type Value = Controller;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("principal byte string")
            }
            fn visit_borrowed_bytes<E: serde::de::Error>(
                self,
                bytes: &'de [u8],
            ) -> Result<Controller, E> {
                Principal::try_from_slice(bytes)
                    .map(Controller)
                    .map_err(E::custom)
            }
        }
        d.deserialize_bytes(Bytes)
    }
}
