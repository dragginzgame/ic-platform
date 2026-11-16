# Mainnet certificate observations

Captured on 2026-11-16 with a read-only HTTPS GET of
`https://a2gzu-ziaaa-aaaam-ae66a-cai.icp0.io/` (ICP documentation canister).
The response was HTTP 200 with HTTP Date `Mon, 16 Nov 2026 11:37:16 GMT`
and `x-ic-canister-id: a2gzu-ziaaa-aaaam-ae66a-cai`.
The GET used a 30-second timeout and a 1 MiB body ceiling. These HTTP headers
are provenance only, not cryptographic trust inputs.

`mainnet.cbor` is exactly the base64-decoded `certificate` field from the
`IC-Certificate` response header. It is a fresh network observation, not copied
source, a third-party test fixture, or a locally signed certificate. Its framing
follows the [HTTP gateway specification](https://docs.internetcomputer.org/references/http-gateway-protocol-spec/).
The HTTP witness/body is not verified or retained; the test authenticates the
state certificate, canister authority and presence of certified data only.

`mainnet-root.der` is the public IC trust anchor, selected independently from the
response. Its 133 bytes match `IC_ROOT_KEY` in registry ic-agent 0.49.3,
[source revision 0857d7dc6a47edbcc754ee41cfc66994ae38b254](https://github.com/dfinity/agent-rs/blob/0857d7dc6a47edbcc754ee41cfc66994ae38b254/ic-agent/src/agent/mod.rs).
Only public key bytes are retained, with no upstream algorithm or test source.
The root remains qualification data; no implicit product root is introduced.

| File | Bytes | SHA-256 |
| --- | --- | --- |
| mainnet.cbor | 2180 | 1ad8ef63fba14c6c9d80b7dcc17b1fa70a75a56ca118b572efedee0c4a975cee |
| mainnet-root.der | 133 | 737ba355e855bd4b61279056603e05501db5e5bad147c6eba7be8c2a13f4b6b3 |

Verified signing time: `1794829035349777381` Unix nanoseconds.
Verified delegation time: `1794828784203552976` Unix nanoseconds.
The observed subnet's 29-byte identity is pinned literally in `mainnet-admission`.
Replays use separately fixed exact historical windows for each timestamp, not
wall-clock freshness or a time inferred from whichever input is being verified.
The initial native capture check used reference `1794829036000000000`, a one-day
maximum age and five-minute future skew for both certificates; those are probe
selections, not product defaults or a live consumer policy recommendation.

Run `make qualify-platform-wasm` to replay on native and standalone Wasm.
The test must not access the network or refresh these fixtures automatically.
A single delegated observation does not establish full mainnet, direct-root,
HTTP asset certification or production consumer qualification.

## Direct NNS observation

`mainnet-direct.cbor` was captured by a read-only GET of `https://nns.ic0.app/`
on 2026-11-16 with the same timeout/body ceiling. The response was HTTP 200,
HTTP Date `Mon, 16 Nov 2026 11:52:11 GMT`, and
`x-ic-canister-id: qoctq-giaaa-aaaaa-aaaea-cai`. Only the base64-decoded
`IC-Certificate` certificate field is retained. It is 623 bytes with SHA-256
`9c2779d17acb459fca060dec5ab4d476d74581491d7424c1c7a8c64386e7021b`.

This certificate verifies directly against the same independently pinned
`mainnet-root.der`, without delegation. Its verified signing time is
`1794829929737402286` Unix nanoseconds. The initial capture check used reference
`1794829931000000000` and five-minute age/future allowances. Maintained replay
uses the fixed exact signing timestamp, not present-day freshness.

The trusted root-subnet binding is selected independently from the
[official system-canister reference](https://docs.internetcomputer.org/references/system-canisters/),
which identifies the NNS UI canister and NNS subnet
`tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe`.
Neither the HTTP routing headers nor the certificate establish this binding.
The test explicitly supplies it and refuses an absent or mismatched binding for
both encoded and parsed selected-subnet projections.

Direct-root authentication has global canister authority. The probe separately
checks that the selected NNS UI canister has a 32-byte certified-data leaf; it
does not manufacture a delegated range check or authenticate the frontend body.
