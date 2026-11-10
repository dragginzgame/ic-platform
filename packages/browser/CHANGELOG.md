# Browser package changelog

This npm package versions independently of the Rust release groups.

## [0.2.0] - 2026-11-05

### ⚠️ Breaking

- Replace the generic `/agent` `Cbor` export with `inspectCallIngress` for typed
  call-envelope inspection and the unchanged SDK request ID. Consumers retain
  byte limits, request-binding, signature/certificate/freshness checks and all
  dispatch policy. Existing envelope bytes and SDK cryptography are unchanged.

### ➕ Added

- Add typed query/read-state inspection with signing metadata and exact paths.
  Provide explicit `/testing` IC response and corruption fixtures so consumer
  signature-metadata, certificate and retained-envelope refusal tests keep their
  coverage without a generic CBOR export.

### 🧭 Migration Notes

- Replace `Cbor.decode(envelope).content` and subsequent request hashing with the
  typed result. See the [Blob migration](../../docs/architecture/migration.md#typed-browser-call-ingress-inspection).
  The browser package versions independently of the Rust release groups.
