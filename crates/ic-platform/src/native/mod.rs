//! IC HTTP transport with explicit trust, limits and time. Anonymous reads are
//! selected by `native-read`; `native-signed` adds explicit Ed25519 operations.
//! `native-signed-secp256k1` also supplies explicit secp256k1 query identities.
//! No Agent, root discovery, cache, retry or polling is selected.
//! Dropping an operation drops its HTTP future; callers own whole-operation deadlines.
use crate::certificate::{
    CertificateDecodeLimits, PreparedRootKey, TimePolicy, VerificationLimits, VerifyError,
    authenticate_certificate, authenticate_subnet_time, decode_certificate,
};
use candidate_types::Principal;
use ic_transport_types::{Envelope, EnvelopeContent, QueryResponse, ReadStateResponse};
use std::{borrow::Cow, collections::BTreeMap, time::Duration};
mod canister;
pub use canister::{CanisterObservation, CanisterReadError};
mod query;
#[cfg(feature = "native-signed")]
mod signed;
#[cfg(feature = "native-signed-secp256k1")]
pub use signed::Secp256k1Identity;
#[cfg(feature = "native-signed")]
pub use signed::{
    AuthenticationProfile, Dispatch, Ed25519Identity, PreparedRequestStatus, PreparedUpdate,
    RequestObservation, RequestStatus, UpdateArgs,
};
mod subnet;
pub use ic_transport_types::SubnetMetrics;
pub use subnet::SubnetObservation;

/// Explicit per-request limits and trust inputs. The caller admits the endpoint/root.
#[derive(Clone, Debug)]
pub struct ReadConfig {
    /// HTTP(S) service origin, without credentials, path, query or fragment.
    pub endpoint: String,
    /// Independently trusted root, never discovered from the endpoint.
    pub root: PreparedRootKey,
    /// Independently trusted root-subnet binding for direct certificates.
    pub root_subnet: Principal,
    /// Complete request and HTTP response byte ceiling.
    pub message_bytes: usize,
    /// HTTP timeout per request; whole multi-request deadlines remain caller-owned.
    pub request_timeout: Duration,
    /// Certificate traversal/refusal limits.
    pub verification: VerificationLimits,
}
/// Typed refusal; successful HTTP or CBOR decoding alone never admits evidence.
#[derive(Debug, thiserror::Error)]
pub enum ReadError {
    /// Invalid endpoint or zero limit/timeout.
    #[error("invalid native read configuration")]
    Configuration,
    /// HTTP, status, timeout or TLS failure.
    #[error("native HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    /// Complete request/response exceeds its byte allowance.
    #[error("native message exceeds byte allowance")]
    ByteLimit,
    /// Request ID, signing identity, envelope signature or CBOR packet failed admission.
    #[error("invalid native packet")]
    Packet,
    /// Certificate bytes failed strict decoding.
    #[error("invalid native certificate: {0}")]
    Decode(#[from] crate::certificate::DecodeError),
    /// Certificate authority/signature/time/resource refusal.
    #[error("native certificate refused: {0}")]
    Verification(#[from] VerifyError),
    /// Selected authenticated path is missing, pruned or mistyped.
    #[error("selected certified path unavailable")]
    MissingPath,
    /// Query node signatures are missing, invalid, duplicated or unauthorized.
    #[error("query node signature refused")]
    QuerySignature,
    /// Query response signature timestamp is outside the explicit window.
    #[error("query response signature time refused")]
    QueryTime,
    /// Authenticated application rejection; code and text are preserved.
    #[error("query rejected ({code}): {message}")]
    Rejected {
        /// Raw source rejection code.
        code: u64,
        /// Raw source rejection message.
        message: String,
        /// Optional raw source error code.
        error_code: Option<String>,
    },
}
/// Reusable bounded client. Signed operations require `native-signed`.
#[derive(Clone)]
pub struct ReadClient {
    config: ReadConfig,
    http: reqwest::Client,
    endpoint: reqwest::Url,
}
impl ReadClient {
    /// Build the bounded client, disabling redirects, proxies and transport retries.
    ///
    /// # Errors
    /// Refuses invalid configuration or HTTP/TLS construction failure.
    pub fn new(config: ReadConfig) -> Result<Self, ReadError> {
        let endpoint =
            reqwest::Url::parse(&config.endpoint).map_err(|_| ReadError::Configuration)?;
        if !matches!(endpoint.scheme(), "http" | "https")
            || endpoint.host_str().is_none()
            || !endpoint.username().is_empty()
            || endpoint.password().is_some()
            || endpoint.path() != "/"
            || endpoint.query().is_some()
            || endpoint.fragment().is_some()
            || config.message_bytes == 0
            || config.request_timeout.is_zero()
        {
            return Err(ReadError::Configuration);
        }
        let http = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .retry(reqwest::retry::never())
            .no_proxy()
            .timeout(config.request_timeout)
            .build()?;
        Ok(Self {
            config,
            http,
            endpoint,
        })
    }
    /// Explicit selected trusted root; no network discovery.
    #[must_use]
    pub const fn root(&self) -> &PreparedRootKey {
        &self.config.root
    }
    async fn post(&self, path: &str, content: &EnvelopeContent) -> Result<Vec<u8>, ReadError> {
        let envelope = Envelope {
            content: Cow::Borrowed(content),
            sender_pubkey: None,
            sender_sig: None,
            sender_delegation: None,
        };
        let body = envelope.encode_bytes();
        self.post_bytes(path, body).await
    }
    async fn post_bytes(&self, path: &str, body: Vec<u8>) -> Result<Vec<u8>, ReadError> {
        self.post_response(path, body).await.map(|(_, bytes)| bytes)
    }
    async fn post_response(
        &self,
        path: &str,
        body: Vec<u8>,
    ) -> Result<(reqwest::StatusCode, Vec<u8>), ReadError> {
        if body.len() > self.config.message_bytes {
            return Err(ReadError::ByteLimit);
        }
        let url = self
            .endpoint
            .join(path)
            .map_err(|_| ReadError::Configuration)?;
        let mut response = self
            .http
            .post(url)
            .header("content-type", "application/cbor")
            .body(body)
            .send()
            .await?
            .error_for_status()?;
        let status = response.status();
        if !status.is_success() {
            return Err(ReadError::Packet);
        }
        if response
            .content_length()
            .is_some_and(|n| n > self.config.message_bytes as u64)
        {
            return Err(ReadError::ByteLimit);
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await? {
            if chunk.len() > self.config.message_bytes.saturating_sub(bytes.len()) {
                return Err(ReadError::ByteLimit);
            }
            bytes.extend_from_slice(&chunk);
        }
        Ok((status, bytes))
    }
    async fn state(
        &self,
        kind: &str,
        target: Principal,
        paths: Vec<Vec<Vec<u8>>>,
        expiry: u64,
    ) -> Result<Vec<u8>, ReadError> {
        let content = EnvelopeContent::ReadState {
            sender: Principal::anonymous()
                .to_string()
                .parse()
                .map_err(|_| ReadError::Packet)?,
            ingress_expiry: expiry,
            paths: paths
                .into_iter()
                .map(|path| path.into_iter().map(Into::into).collect())
                .collect(),
        };
        let bytes = self
            .post(&format!("api/v3/{kind}/{target}/read_state"), &content)
            .await?;
        let response: ReadStateResponse =
            serde_cbor::from_slice(&bytes).map_err(|_| ReadError::Packet)?;
        Ok(response.certificate)
    }
    fn decode<'a>(
        &self,
        bytes: &'a [u8],
    ) -> Result<crate::certificate::DecodedCertificate<'a>, ReadError> {
        Ok(decode_certificate(
            bytes,
            CertificateDecodeLimits {
                bytes: self.config.message_bytes,
            },
        )?)
    }
}

#[cfg(all(test, feature = "native-signed", feature = "certificate-construction"))]
mod signed_tests;

#[cfg(all(test, feature = "certificate-fixture-signing"))]
mod canister_tests;
