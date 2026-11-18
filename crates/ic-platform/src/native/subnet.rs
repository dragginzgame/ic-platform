use super::{
    BTreeMap, Principal, ReadClient, ReadError, SubnetMetrics, TimePolicy, authenticate_subnet_time,
};
use ic_certification::LookupResult;

/// Immutable selected-subnet observation authenticated under explicit root/time inputs.
/// Self-reported metadata never establishes canister routing authority.
#[derive(Debug)]
pub struct SubnetObservation {
    subnet: Principal,
    certificate: Vec<u8>,
    time_ns: u64,
    kind: Option<String>,
    metrics: Option<SubnetMetrics>,
    pub(super) node_keys: BTreeMap<Vec<u8>, Vec<u8>>,
}
impl SubnetObservation {
    /// Exact authenticated response certificate bytes.
    #[must_use]
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }

    /// Exact selected subnet identity, authenticated independently of effective canisters.
    #[must_use]
    pub const fn subnet(&self) -> Principal {
        self.subnet
    }

    /// Authenticated signing time in nanoseconds.
    #[must_use]
    pub const fn time_ns(&self) -> u64 {
        self.time_ns
    }

    /// Raw certified subnet type. Unknown strings remain visible; absence is None.
    #[must_use]
    pub fn subnet_type(&self) -> Option<&str> {
        self.kind.as_deref()
    }

    /// All four raw certified metrics, or None when the selected leaf is absent.
    #[must_use]
    pub const fn metrics(&self) -> Option<&SubnetMetrics> {
        self.metrics.as_ref()
    }

    /// Certified node key bytes; this is not application enrollment.
    pub fn node_keys(&self) -> impl Iterator<Item = (&[u8], &[u8])> {
        self.node_keys
            .iter()
            .map(|(id, key)| (id.as_slice(), key.as_slice()))
    }
}
impl ReadClient {
    /// Fetch selected-subnet metadata and authenticate that exact subnet's authority.
    /// No ranges returned by this subnet are used to infer canister routing.
    ///
    /// # Errors
    /// Refuses transport, framing, authority/time, missing-path and malformed metadata.
    pub async fn read_subnet(
        &self,
        subnet: Principal,
        expiry: u64,
        time: TimePolicy,
    ) -> Result<SubnetObservation, ReadError> {
        let bytes = self
            .state(
                "subnet",
                subnet,
                vec![
                    vec![b"subnet".to_vec(), subnet.as_slice().to_vec()],
                    vec![b"time".to_vec()],
                ],
                expiry,
            )
            .await?;
        self.admit_subnet(bytes, subnet, time)
    }

    pub(super) fn admit_subnet(
        &self,
        bytes: Vec<u8>,
        subnet: Principal,
        time: TimePolicy,
    ) -> Result<SubnetObservation, ReadError> {
        let decoded = self.decode(&bytes)?;
        let evidence = authenticate_subnet_time(
            self.decode(&bytes)?,
            &self.config.root,
            subnet,
            Some(self.config.root_subnet),
            time,
            self.config.verification,
        )?;
        let time_ns = evidence.time_ns();
        let tree = &decoded.unverified().tree;
        let prefix = [b"subnet".as_slice(), subnet.as_slice()];
        let optional = |name: &[u8]| -> Result<Option<&[u8]>, ReadError> {
            match tree.lookup_path([prefix[0], prefix[1], name]) {
                LookupResult::Found(bytes) => Ok(Some(bytes)),
                LookupResult::Absent => Ok(None),
                _ => Err(ReadError::MissingPath),
            }
        };
        let kind = optional(b"type")?
            .map(|bytes| {
                std::str::from_utf8(bytes)
                    .map(str::to_owned)
                    .map_err(|_| ReadError::Packet)
            })
            .transpose()?;
        let metrics = optional(b"metrics")?
            .map(|bytes| serde_cbor::from_slice(bytes).map_err(|_| ReadError::Packet))
            .transpose()?;
        let mut node_keys = BTreeMap::new();
        for path in tree.list_paths() {
            if path.len() == 5
                && path[0].as_bytes() == b"subnet"
                && path[1].as_bytes() == subnet.as_slice()
                && path[2].as_bytes() == b"node"
                && path[4].as_bytes() == b"public_key"
            {
                Principal::try_from_slice(path[3].as_bytes()).map_err(|_| ReadError::Packet)?;
                let LookupResult::Found(key) = tree.lookup_path(path.iter().map(|p| p.as_bytes()))
                else {
                    return Err(ReadError::MissingPath);
                };
                node_keys.insert(path[3].as_bytes().to_vec(), key.to_vec());
            }
        }
        Ok(SubnetObservation {
            subnet,
            certificate: bytes,
            time_ns,
            kind,
            metrics,
            node_keys,
        })
    }
}
