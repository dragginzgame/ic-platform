use super::{
    EnvelopeContent, Principal, QueryResponse, ReadClient, ReadError, SubnetObservation,
    TimePolicy, authenticate_certificate,
};
use crate::ed25519::{Signature, VerifyingKey};
use std::collections::BTreeSet;
impl ReadClient {
    /// Execute one anonymous query and authenticate every response node signature.
    /// Routing authority comes from a canister-scoped certificate's root-certified
    /// delegation ranges, then a separately authenticated selected-subnet key read.
    /// The response is node-authenticated, not a consensus-certified query result.
    ///
    /// # Errors
    /// Refuses transport/resource/packet failures, authority, signature, timestamp
    /// or node-key failures, and authenticated application rejection.
    pub async fn query(
        &self,
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
        let content = EnvelopeContent::Query {
            sender: Principal::anonymous()
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
        self.query_content(canister, &content, expiry, time, None)
            .await
    }
    pub(super) async fn query_content(
        &self,
        canister: Principal,
        content: &EnvelopeContent,
        expiry: u64,
        time: TimePolicy,
        signed: Option<Vec<u8>>,
    ) -> Result<Vec<u8>, ReadError> {
        let request_id =
            ic_transport_types::to_request_id(content).map_err(|_| ReadError::Packet)?;
        let path = format!("api/v3/canister/{canister}/query");
        let bytes = match signed {
            Some(body) => self.post_bytes(&path, body).await?,
            None => self.post(&path, content).await?,
        };
        let response: QueryResponse =
            serde_cbor::from_slice(&bytes).map_err(|_| ReadError::Packet)?;
        // No caller-supplied subnet or self-reported range can authorize a node.
        let certificate = self
            .state("canister", canister, vec![vec![b"time".to_vec()]], expiry)
            .await?;
        let evidence = authenticate_certificate(
            self.decode(&certificate)?,
            &self.config.root,
            canister,
            time,
            self.config.verification,
        )?;
        let subnet = evidence
            .delegation()
            .map_or(self.config.root_subnet, |(id, _, _)| id);
        let nodes = self.read_subnet(subnet, expiry, time).await?;
        admit_response(response, request_id, &nodes, time.certificate)
    }
}
fn admit_response(
    response: QueryResponse,
    request_id: ic_transport_types::RequestId,
    nodes: &SubnetObservation,
    time: crate::certificate::TimeWindow,
) -> Result<Vec<u8>, ReadError> {
    let minimum = time
        .reference_ns
        .checked_sub(time.max_age_ns)
        .ok_or(ReadError::QueryTime)?;
    let maximum = time
        .reference_ns
        .checked_add(time.max_future_skew_ns)
        .ok_or(ReadError::QueryTime)?;
    let signatures = response.signatures();
    if signatures.is_empty() || signatures.len() > nodes.node_keys.len() {
        return Err(ReadError::QuerySignature);
    }
    let mut seen = BTreeSet::new();
    for signature in signatures {
        if signature.timestamp < minimum || signature.timestamp > maximum {
            return Err(ReadError::QueryTime);
        }
        if !seen.insert(signature.identity.as_slice()) {
            return Err(ReadError::QuerySignature);
        }
        let bytes = nodes
            .node_keys
            .get(signature.identity.as_slice())
            .ok_or(ReadError::QuerySignature)?;
        // RFC 8410 Ed25519 SPKI, exact OID and byte-aligned 32-byte key.
        const PREFIX: &[u8] = b"\x30\x2a\x30\x05\x06\x03\x2b\x65\x70\x03\x21\x00";
        let raw: &[u8; 32] = bytes
            .strip_prefix(PREFIX)
            .and_then(|b| b.try_into().ok())
            .ok_or(ReadError::QuerySignature)?;
        let key = VerifyingKey::from_bytes(raw).map_err(|_| ReadError::QuerySignature)?;
        if key.is_weak() {
            return Err(ReadError::QuerySignature);
        }
        let sig =
            Signature::from_slice(&signature.signature).map_err(|_| ReadError::QuerySignature)?;
        key.verify_strict(&response.signable(request_id, signature.timestamp), &sig)
            .map_err(|_| ReadError::QuerySignature)?;
    }
    match response {
        QueryResponse::Replied { reply, .. } => Ok(reply.arg),
        QueryResponse::Rejected { reject, .. } => Err(ReadError::Rejected {
            code: reject.reject_code as u64,
            message: reject.reject_message,
            error_code: reject.error_code,
        }),
    }
}

#[cfg(all(test, feature = "certificate-construction"))]
mod tests;
