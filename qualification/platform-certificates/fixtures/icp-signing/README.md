# ICP CLI 1.7.0 direct-key envelopes

These public signed fixtures were produced by the original official Linux
x86-64 ICP CLI 1.7.0 binary. `provenance.json` records the verified release
archive checksum, exact envelope hashes and CLI-reported original request IDs.
The official archive and its `.sha256` asset were downloaded from
https://github.com/dfinity/icp-cli/releases/tag/v1.7.0.

Keys were generated only for this offline test with upstream OpenSSL, imported
into an isolated `ICP_HOME`/`XDG_CONFIG_HOME` with plaintext storage, and never
committed. No real identity or canister was used. For each Ed25519/secp256k1 key:

```sh
icp identity import PROFILE --from-pem TEST_KEY --storage plaintext
icp canister call aaaaa-aa canister_status 4449444c0000 --args-format hex \
  --network http://127.0.0.1:1 --root-key mainnet --identity PROFILE \
  --sign-only OUTPUT.json
```

The binary `update` and `status` files are base64-decoded `request.envelope` and
`request.status_check` from that output. No CBOR normalization was applied. ICP
refuses an already closed `--valid-from` window; fixtures used the current window
when generated. Loading retained envelopes intentionally continues to work after
expiry. Nothing was submitted to the IC or the selected unreachable endpoint.
The empty argument frame is only a signing fixture, not a valid status invocation.

`native_signed` tests load these public bytes in a fresh process, authenticate a
synthetic BLS-certified original result from a loopback server, and assert the
exact external status envelope is sent once. Independent product tests exercise
malformed/signature/sender/path/bound refusals. This is supplier interoperability,
not Canic's complete durable operation journal or Blob's CLI lifecycle acceptance.
Unsupported P-256/delegated profiles still refuse; no broad ICP identity claim is
made. Signature verification uses original upstream implementations, not ICP code
copied into Platform.
