# Changelog

## [0.1.2]

### Added

- Add `make publish` for explicit crates.io publication of the current committed
  package version, and `make publish-check` for an all-feature Cargo dry run.
  Retain command output and source identity for each attempt.

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
