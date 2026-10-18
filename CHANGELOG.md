# Changelog

## [0.1.3]

### Added

- Export `ic_platform::Principal` unconditionally, preserving the Candid
  contract's upstream type identity. Identity-only consumers need no feature or
  direct `ic_principal` dependency; default builds remain Candid/CDK-free.
- Add an independent optional `certification` surface for upstream certificate
  and hash-tree types and operations used by Host, Query and Canic. Preserve
  upstream type identity and Serde support; bounded decoding and verification
  policy remain consumer-owned. Qualify a standalone certification consumer.

## [0.1.2] - 2026-10-18

### Added

- Add `make publish` for explicit crates.io publication of the current committed
  package version, and `make publish-check` for an all-feature Cargo dry run.
  Retain command output and source identity for each attempt.

### Changed

- Adopt Shared Tooling 0.24.0 with production dependency-role labels, matching
  formatter/release ordering checks and the shared formatting hook payload.
- Check host-parser and Testkit exclusions transitively across every supported
  feature selection on native and Wasm targets. Deployment artifact admission
  remains the responsibility of consuming applications.

## [0.1.1] - 2026-10-18

### Added

- Support Testkit/Query generic argument helpers and external arbitrary-precision
  and reserved values through the explicit Candid boundary.
- Extend standalone consumer qualification to Rust 1.88, recursive DTOs,
  independent wire vectors and typed decoder causes, with Wasm compilation.

### Changed

- Document consumer migration and decoder retirement requirements, including
  the remaining CDK runtime and host-parser adoption boundaries.
- Check native/Wasm executor dependency graphs and parser isolation while
  preserving empty default features and Candid-free executor-only builds.

## [0.1.0]

### Added

- Establish the controlled IC platform boundary with no default capabilities.
- Adopt Testkit's caller-bounded Candid decoding, preserving exact-result and
  argument-list policies, compact diagnostics and typed causes.
- Expose DTO derives and encoding through an explicit external-contract surface.
- Expose the shared CDK executor independently of Candid. No full CDK runtime,
  host parser or consumer migration is included in this initial local candidate.
