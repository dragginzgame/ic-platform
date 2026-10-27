//! Standalone native asset contract consumer; all acceptance policy remains here.
#![cfg(not(target_family = "wasm"))]
#[cfg(test)]
mod tests {
    use ic_platform::{
        candid::{DecodeLimits, contract, decode_exactly_one},
        certified_assets_types::*,
    };
    fn limits() -> DecodeLimits {
        DecodeLimits {
            reply_bytes: 4096,
            decoding_quota: 100_000,
            skipping_quota: 0,
            type_entries: 32,
            header_bytes: 2048,
        }
    }
    // A test consumer's selected acceptance policy, never a Platform default.
    fn admit(reply: &GetResult, digest: &[u8], maximum: u64) -> bool {
        reply.content_encoding == "identity"
            && reply.sha256.as_deref() == Some(digest)
            && reply.total_length <= maximum
            && reply.content.len() as u64 <= maximum
    }
    #[test]
    fn asset_first_chunk_and_consumer_owned_acceptance() {
        let request = GetArgs {
            key: "/index.html".into(),
            accept_encodings: vec!["identity".into()],
        };
        let bytes = contract::encode_one(&request).unwrap();
        assert_eq!(
            decode_exactly_one::<GetArgs>(&bytes, limits()).unwrap(),
            request
        );
        let digest = vec![7; 32];
        let mut response = GetResult {
            content: vec![0, 255],
            content_type: "text/html".into(),
            content_encoding: "identity".into(),
            sha256: Some(digest.clone()),
            total_length: 3u64.into(),
        };
        let bytes = contract::encode_one(&response).unwrap();
        let decoded = decode_exactly_one::<GetResult>(&bytes, limits()).unwrap();
        assert_eq!(decoded, response);
        assert!(admit(&decoded, &digest, 3));
        assert!(!admit(&decoded, &digest, 2));
        assert!(!admit(&decoded, &[8; 32], 3));
        response.sha256 = None;
        assert!(!admit(&response, &digest, 3));
        response.sha256 = Some(digest.clone());
        response.content_encoding = "gzip".into();
        assert!(!admit(&response, &digest, 3));
        assert!(
            decode_exactly_one::<GetResult>(
                &bytes,
                DecodeLimits {
                    reply_bytes: bytes.len() - 1,
                    ..limits()
                }
            )
            .is_err()
        );
        assert!(
            decode_exactly_one::<GetResult>(
                &bytes,
                DecodeLimits {
                    decoding_quota: 0,
                    ..limits()
                }
            )
            .is_err()
        );
        assert!(decode_exactly_one::<GetResult>(&bytes[..bytes.len() - 1], limits()).is_err());
        let request = GetChunkArgs {
            key: "/index.html".into(),
            content_encoding: "identity".into(),
            index: 1u64.into(),
            sha256: Some(digest),
        };
        let bytes = contract::encode_one(&request).unwrap();
        assert_eq!(
            decode_exactly_one::<GetChunkArgs>(&bytes, limits()).unwrap(),
            request
        );
        let response = GetChunkResult { content: vec![42] };
        let bytes = contract::encode_one(&response).unwrap();
        assert_eq!(
            decode_exactly_one::<GetChunkResult>(&bytes, limits()).unwrap(),
            response
        );
    }
}
