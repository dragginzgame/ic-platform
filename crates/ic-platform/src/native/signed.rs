//! Explicit Ed25519 identity and original-request transport. No discovery,
//! polling, renewal, redirect, proxy or retry is performed by these operations.
use super::{ReadClient, ReadError, TimePolicy, authenticate_certificate};
use crate::{
    ed25519::{Signature, Signer, SigningKey, VerifyingKey},
    ingress::{self, Content, Envelope},
};
use candidate_types::Principal;
use ed25519_dalek::pkcs8::DecodePrivateKey;
use ic_certification::LookupResult;
use serde::Serialize;

/// Explicit original update fields; construction performs no network activity.
#[derive(Clone, Copy, Debug)]
pub struct UpdateArgs<'a> {
    /// Signed destination, possibly the management canister.
    pub recipient: Principal,
    /// HTTP routing and certificate-authority target.
    pub effective: Principal,
    /// Exact method name.
    pub method: &'a str,
    /// Exact argument bytes.
    pub arg: &'a [u8],
    /// Original expiry in nanoseconds.
    pub expiry: u64,
    /// Optional nonce, preserving absence versus present empty bytes.
    pub nonce: Option<&'a [u8]>,
}
/// Caller-supplied Ed25519 key. It is never discovered or loaded from disk here.
/// Secret storage is zeroized by the upstream signing key on drop.
pub struct Ed25519Identity(SigningKey);
impl Ed25519Identity {
    /// Parse an explicit PKCS#8 Ed25519 PEM (BasicIdentity format), offline.
    /// # Errors
    /// Refuses PEM exceeding the inclusive bound, another algorithm or invalid key.
    pub fn from_pem(pem: &str, maximum_bytes: usize) -> Result<Self, ReadError> {
        if pem.len() > maximum_bytes {
            return Err(ReadError::ByteLimit);
        }
        SigningKey::from_pkcs8_pem(pem)
            .map(Self)
            .map_err(|_| ReadError::Packet)
    }
    fn public_key(&self) -> Vec<u8> {
        [
            b"\x30\x2a\x30\x05\x06\x03\x2b\x65\x70\x03\x21\x00".as_slice(),
            &self.0.verifying_key().to_bytes(),
        ]
        .concat()
    }
    /// Offline canonical self-authenticating identity.
    #[must_use]
    pub fn principal(&self) -> Principal {
        Principal::self_authenticating(self.public_key())
    }
    fn envelope(&self, content: Content, maximum: usize) -> Result<Vec<u8>, ReadError> {
        let signature = self
            .0
            .sign(&content.signable().map_err(|_| ReadError::Packet)?);
        encode(
            &Envelope {
                content,
                sender_pubkey: Some(self.public_key()),
                sender_sig: Some(signature.to_bytes().to_vec()),
                sender_delegation: None,
            },
            maximum,
        )
    }
    /// Prepare an exact signed update offline, with explicit expiry and nonce.
    /// Recipient and effective routing target are intentionally distinct.
    /// # Errors
    /// Refuses oversized fields/envelope or packet/signing admission failure.
    pub fn prepare_update(
        &self,
        args: UpdateArgs<'_>,
        maximum_bytes: usize,
    ) -> Result<PreparedUpdate, ReadError> {
        let UpdateArgs {
            recipient,
            effective,
            method,
            arg,
            expiry,
            nonce,
        } = args;
        if method
            .len()
            .checked_add(arg.len())
            .and_then(|n| n.checked_add(nonce.map_or(0, <[u8]>::len)))
            .is_none_or(|n| n > maximum_bytes)
        {
            return Err(ReadError::ByteLimit);
        }
        let bytes = self.envelope(
            Content::Call {
                nonce: nonce.map(<[u8]>::to_vec),
                ingress_expiry: expiry,
                sender: self.principal(),
                canister_id: recipient,
                method_name: method.to_owned(),
                arg: arg.to_vec(),
                sender_info: None,
            },
            maximum_bytes,
        )?;
        PreparedUpdate::from_bytes(&bytes, effective, maximum_bytes)
    }
}
// Two passes admit complete serialized output before allocating the buffer.
fn encode<T: Serialize>(value: &T, maximum: usize) -> Result<Vec<u8>, ReadError> {
    struct Count {
        count: usize,
        maximum: usize,
    }
    impl std::io::Write for Count {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.count = self
                .count
                .checked_add(bytes.len())
                .filter(|n| *n <= self.maximum)
                .ok_or_else(|| std::io::Error::other("message limit"))?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut count = Count { count: 0, maximum };
    serde_cbor::to_writer(&mut count, value).map_err(|_| ReadError::ByteLimit)?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(count.count)
        .map_err(|_| ReadError::ByteLimit)?;
    serde_cbor::to_writer(&mut bytes, value).map_err(|_| ReadError::Packet)?;
    Ok(bytes)
}
/// Immutable, signature-checked original update. Loading does not check expiry:
/// recovery must remain possible after the original ingress has expired.
#[derive(Debug)]
pub struct PreparedUpdate {
    bytes: Vec<u8>,
    id: [u8; 32],
    expiry: u64,
    sender: Principal,
    recipient: Principal,
    effective: Principal,
}
impl PreparedUpdate {
    /// Admit exact retained bytes from durable custody; never re-encode them.
    /// Only direct Ed25519 envelopes are selected. Caller admits effective routing.
    /// # Errors
    /// Refuses malformed/oversized envelopes, delegation, sender info, wrong sender
    /// binding, weak keys or invalid signatures. No freshness claim is made.
    pub fn from_bytes(
        bytes: &[u8],
        effective: Principal,
        maximum_bytes: usize,
    ) -> Result<Self, ReadError> {
        let envelope = ingress::decode_envelope(bytes, maximum_bytes)
            .map_err(|_| ReadError::Packet)?
            .value;
        let Content::Call {
            ingress_expiry,
            sender,
            canister_id,
            ref sender_info,
            ..
        } = envelope.content
        else {
            return Err(ReadError::Packet);
        };
        if envelope.sender_delegation.is_some() || sender_info.is_some() {
            return Err(ReadError::Packet);
        }
        let public = envelope.sender_pubkey.as_deref().ok_or(ReadError::Packet)?;
        let raw: &[u8; 32] = public
            .strip_prefix(b"\x30\x2a\x30\x05\x06\x03\x2b\x65\x70\x03\x21\x00")
            .and_then(|p| p.try_into().ok())
            .ok_or(ReadError::Packet)?;
        let key = VerifyingKey::from_bytes(raw).map_err(|_| ReadError::Packet)?;
        if key.is_weak() || Principal::self_authenticating(public) != sender {
            return Err(ReadError::Packet);
        }
        let signature =
            Signature::from_slice(envelope.sender_sig.as_deref().ok_or(ReadError::Packet)?)
                .map_err(|_| ReadError::Packet)?;
        key.verify_strict(
            &envelope.content.signable().map_err(|_| ReadError::Packet)?,
            &signature,
        )
        .map_err(|_| ReadError::Packet)?;
        Ok(Self {
            bytes: bytes.to_vec(),
            id: envelope
                .content
                .request_id()
                .map_err(|_| ReadError::Packet)?,
            expiry: ingress_expiry,
            sender,
            recipient: canister_id,
            effective,
        })
    }
    /// Exact original envelope bytes, including original CBOR framing.
    #[must_use]
    pub fn envelope_bytes(&self) -> &[u8] {
        &self.bytes
    }
    /// Original representation-independent request identity.
    #[must_use]
    pub const fn request_id(&self) -> &[u8; 32] {
        &self.id
    }
    /// Original expiry, never renewed by observation.
    #[must_use]
    pub const fn expiry(&self) -> u64 {
        self.expiry
    }
    /// Signed sender identity.
    #[must_use]
    pub const fn sender(&self) -> Principal {
        self.sender
    }
    /// Signed recipient (possibly the management canister).
    #[must_use]
    pub const fn recipient(&self) -> Principal {
        self.recipient
    }
    /// Explicit HTTP route and certificate authority target.
    #[must_use]
    pub const fn effective_canister(&self) -> Principal {
        self.effective
    }
}
/// Certified observation of exactly the original request. Absence and pruning
/// are not evidence of nonexecution and never authorize replacement submission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RequestStatus {
    /// Certified `received` state.
    Received,
    /// Certified `processing` state.
    Processing,
    /// Certified exact reply bytes.
    Replied(Vec<u8>),
    /// Certified rejection with raw numeric code and diagnostic text.
    Rejected {
        /// Raw source code.
        code: u64,
        /// Exact message.
        message: String,
        /// Optional exact source error code.
        error_code: Option<String>,
    },
    /// Certified terminal marker; reply has been pruned.
    Done,
    /// Certified absence of the selected request status.
    Unknown,
    /// The witness does not reveal the selected path.
    Unavailable,
}
/// Immutable verified status with the original certificate and request binding.
#[derive(Debug)]
pub struct RequestObservation {
    certificate: Vec<u8>,
    id: [u8; 32],
    effective: Principal,
    time_ns: u64,
    status: RequestStatus,
}
impl RequestObservation {
    /// Exact proof bytes, retained without re-encoding.
    #[must_use]
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    /// Original request selected by this observation.
    #[must_use]
    pub const fn request_id(&self) -> &[u8; 32] {
        &self.id
    }
    /// Authenticated effective target.
    #[must_use]
    pub const fn effective_canister(&self) -> Principal {
        self.effective
    }
    /// Certified observation time.
    #[must_use]
    pub const fn time_ns(&self) -> u64 {
        self.time_ns
    }
    /// Authenticated selected outcome.
    #[must_use]
    pub const fn status(&self) -> &RequestStatus {
        &self.status
    }
}
/// Dispatch response. An error from dispatch is delivery uncertainty, never
/// evidence that the original request was not executed.
#[derive(Debug)]
pub enum Dispatch {
    /// Accepted without completion evidence.
    Accepted,
    /// Authenticated original-request observation.
    Observed(RequestObservation),
    /// Uncertified boundary-node rejection; not an executed certified rejection.
    UncertifiedRejection {
        /// Raw numeric code.
        code: u64,
        /// Exact diagnostic.
        message: String,
        /// Optional exact error code.
        error_code: Option<String>,
    },
}
impl ReadClient {
    /// Send the retained original bytes exactly once. No redirect/retry/polling.
    /// Caller owns custody, expiry admission and any later observation policy.
    /// # Errors
    /// Every transport/status/packet/proof refusal leaves delivery uncertain.
    pub async fn dispatch(
        &self,
        update: &PreparedUpdate,
        time: TimePolicy,
    ) -> Result<Dispatch, ReadError> {
        if update.bytes.len() > self.config.message_bytes {
            return Err(ReadError::ByteLimit);
        }
        let (status, bytes) = self
            .post_response(
                &format!("api/v3/canister/{}/call", update.effective),
                update.bytes.clone(),
            )
            .await?;
        if bytes.is_empty() && status == reqwest::StatusCode::ACCEPTED {
            return Ok(Dispatch::Accepted);
        }
        match ingress::decode_call_response(&bytes, self.config.message_bytes)
            .map_err(|_| ReadError::Packet)?
            .value
        {
            ingress::CallResponse::Accepted => Ok(Dispatch::Accepted),
            ingress::CallResponse::Replied { certificate } => self
                .authenticate_request(update, &certificate, time)
                .map(Dispatch::Observed),
            ingress::CallResponse::NonReplicatedRejection {
                reject_code,
                reject_message,
                error_code,
            } => Ok(Dispatch::UncertifiedRejection {
                code: reject_code,
                message: reject_message,
                error_code,
            }),
        }
    }
    /// Observe the original ID once using a newly signed read-state request.
    /// `read_expiry` applies only to that read; the update is never renewed/sent.
    /// `time` admits current network evidence independently of original expiry.
    /// # Errors
    /// Refuses wrong identity, transport or certificate admission failures.
    pub async fn observe_request(
        &self,
        identity: &Ed25519Identity,
        update: &PreparedUpdate,
        read_expiry: u64,
        time: TimePolicy,
    ) -> Result<RequestObservation, ReadError> {
        if identity.principal() != update.sender {
            return Err(ReadError::Packet);
        }
        let body = identity.envelope(
            Content::ReadState {
                ingress_expiry: read_expiry,
                sender: update.sender,
                paths: vec![
                    vec![b"request_status".to_vec(), update.id.to_vec()],
                    vec![b"time".to_vec()],
                ],
            },
            self.config.message_bytes,
        )?;
        let bytes = self
            .post_bytes(
                &format!("api/v3/canister/{}/read_state", update.effective),
                body,
            )
            .await?;
        let response = ingress::decode_read_state_response(&bytes, self.config.message_bytes)
            .map_err(|_| ReadError::Packet)?;
        self.authenticate_request(update, &response.value.certificate, time)
    }
    /// Replay an original proof offline under explicitly selected historical
    /// certificate/delegation time windows. No current-network freshness inferred.
    /// # Errors
    /// Refuses wrong authority/root, malformed proofs or selected status leaves.
    pub fn authenticate_request(
        &self,
        update: &PreparedUpdate,
        certificate: &[u8],
        time: TimePolicy,
    ) -> Result<RequestObservation, ReadError> {
        let evidence = authenticate_certificate(
            self.decode(certificate)?,
            &self.config.root,
            update.effective,
            time,
            self.config.verification,
        )?;
        let tree = evidence.tree();
        let lookup = |name: &[u8]| {
            tree.lookup_path([b"request_status".as_slice(), update.id.as_slice(), name])
        };
        let required = |name: &[u8]| match lookup(name) {
            LookupResult::Found(bytes) => Ok(bytes),
            _ => Err(ReadError::MissingPath),
        };
        let text = |name: &[u8]| {
            std::str::from_utf8(required(name)?)
                .map(str::to_owned)
                .map_err(|_| ReadError::Packet)
        };
        let status = match lookup(b"status") {
            LookupResult::Absent => RequestStatus::Unknown,
            LookupResult::Unknown => RequestStatus::Unavailable,
            LookupResult::Error => return Err(ReadError::Packet),
            LookupResult::Found(status) => match status {
                b"received" => RequestStatus::Received,
                b"processing" => RequestStatus::Processing,
                b"done" => RequestStatus::Done,
                b"replied" => RequestStatus::Replied(required(b"reply")?.to_vec()),
                b"rejected" => RequestStatus::Rejected {
                    code: unsigned(required(b"reject_code")?)?,
                    message: text(b"reject_message")?,
                    error_code: match lookup(b"error_code") {
                        LookupResult::Absent => None,
                        LookupResult::Found(value) => Some(
                            std::str::from_utf8(value)
                                .map_err(|_| ReadError::Packet)?
                                .to_owned(),
                        ),
                        _ => return Err(ReadError::MissingPath),
                    },
                },
                _ => return Err(ReadError::Packet),
            },
        };
        Ok(RequestObservation {
            certificate: certificate.to_vec(),
            id: update.id,
            effective: update.effective,
            time_ns: evidence.time_ns(),
            status,
        })
    }
    /// Execute one explicitly signed query and authenticate its node signatures
    /// through the existing certified node-key/routing path. Caller bounds the
    /// whole operation in addition to the configured per-request timeout.
    /// # Errors
    /// Refuses oversized requests, transport, node authority/signature/time failures.
    pub async fn signed_query(
        &self,
        identity: &Ed25519Identity,
        canister: Principal,
        method: &str,
        args: &[u8],
        expiry: u64,
        time: TimePolicy,
    ) -> Result<Vec<u8>, ReadError> {
        if args
            .len()
            .checked_add(method.len())
            .is_none_or(|n| n > self.config.message_bytes)
        {
            return Err(ReadError::ByteLimit);
        }
        let content = ic_transport_types::EnvelopeContent::Query {
            sender: identity
                .principal()
                .to_string()
                .parse()
                .map_err(|_| ReadError::Packet)?,
            canister_id: canister
                .to_string()
                .parse()
                .map_err(|_| ReadError::Packet)?,
            ingress_expiry: expiry,
            method_name: method.to_owned(),
            arg: args.to_vec(),
            nonce: None,
            sender_info: None,
        };
        let id = ic_transport_types::to_request_id(&content).map_err(|_| ReadError::Packet)?;
        let envelope = ic_transport_types::Envelope {
            content: std::borrow::Cow::Borrowed(&content),
            sender_pubkey: Some(identity.public_key()),
            sender_sig: Some(identity.0.sign(&id.signable()).to_bytes().to_vec()),
            sender_delegation: None,
        };
        self.query_content(
            canister,
            &content,
            expiry,
            time,
            Some(encode(&envelope, self.config.message_bytes)?),
        )
        .await
    }
}
fn unsigned(bytes: &[u8]) -> Result<u64, ReadError> {
    let mut value = 0;
    for (index, byte) in bytes.iter().copied().enumerate() {
        if index > 9 || (index == 9 && byte > 1) {
            return Err(ReadError::Packet);
        }
        value |= u64::from(byte & 127) << (index * 7);
        if byte & 128 == 0 {
            return if index + 1 == bytes.len() {
                Ok(value)
            } else {
                Err(ReadError::Packet)
            };
        }
    }
    Err(ReadError::Packet)
}

impl ReadClient {
    /// Read and authenticate the exact installed module hash. This observation
    /// is not atomic with a later application call or an upgrade authorization.
    /// # Errors
    /// Refuses transport, authority/time failures, absent/pruned or non-32-byte hash.
    pub async fn canister_module_hash(
        &self,
        canister: Principal,
        expiry: u64,
        time: TimePolicy,
    ) -> Result<[u8; 32], ReadError> {
        let bytes = self
            .state(
                "canister",
                canister,
                vec![
                    vec![
                        b"canister".to_vec(),
                        canister.as_slice().to_vec(),
                        b"module_hash".to_vec(),
                    ],
                    vec![b"time".to_vec()],
                ],
                expiry,
            )
            .await?;
        let evidence = authenticate_certificate(
            self.decode(&bytes)?,
            &self.config.root,
            canister,
            time,
            self.config.verification,
        )?;
        match evidence.tree().lookup_path([
            b"canister".as_slice(),
            canister.as_slice(),
            b"module_hash".as_slice(),
        ]) {
            LookupResult::Found(hash) => hash.try_into().map_err(|_| ReadError::Packet),
            _ => Err(ReadError::MissingPath),
        }
    }
}
