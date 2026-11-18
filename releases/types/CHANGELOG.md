# Types changelog

Release group: `types`. Covered package: `ic-platform-types`.
Return to the [group index](../../CHANGELOG.md).

## [0.21.3] - 2026-11-18

### 🔧 Maintenance

- Move Account and Principal tests to the top of their module declarations under the updated Rust layout policy.

## [0.21.2] - 2026-11-16

### 🩹 Fixed

- Replace the exact thiserror 2.0.18 pin with a compatible requirement. Types
  no longer forces consumers onto that patch; its implementation qualifies
  with 2.0.21 while constrained qualification graphs may retain 2.0.18.

### 🔧 Tooling

- Adopt Toko Tooling 0.44.9 in both workspaces and use its flat qualification
  package layout. Remove the superseded fleet-wide thiserror hold.

### 🧪 Testing

- Update the qualification registry Types selection to 0.21.1 alongside the local
  0.21.1 candidate, retaining explicit Cargo source identity boundaries.

- Qualify published Types 0.21.1 with Wire 0.24.2 through Auth's actual generator,
  retained browser frames and unchanged generated client output, resolving
  [#8 — Types/Runtime and Wire cohort for Auth](https://github.com/dragginzgame/ic-platform/issues/8).
  This is qualification-only work; the Types package remains unchanged at 0.21.1.

## [0.21.1] - 2026-11-16

### 📦 Release preparation

- Prepare the first independent Types release with repository metadata, packaged
  documentation and an explicit crates.io publisher selection. Production
  migration remains a separate gate, as approved by the maintainer.
- Qualify the packaged crate through pinned Auth passive model source on native
  and Wasm; extend published-oracle checks to all Principal lengths and each
  Subaccount bit, including typed malformed-Principal refusals.
- Preserve empty defaults, opt-in self-authentication and one source/type identity.

## [0.21.0] - Historical GitHub source candidate

Initial local cohort candidate after registry Types 0.20.4. This incomplete
pre-1.0 implementation is unpublished and must not replace the published cohort
yet. Full production consumer migration and qualification remain required.

### ➕ Added

- Restore opt-in `principal-self-authenticating` using upstream SHA-224 and the
  retained identity tag, without enabling external identity conversions.

- Establish the actual local Types package and independent release-group owner.
- Add Account/Subaccount with effective-zero identity, presence-preserving Serde,
  bounded text admission and caller-buffer text rendering. Retain checksum
  framing and malformed-input refusal precedence with independent fixtures.
- Expose `Principal::write_text` for canonical text in caller-owned fixed storage,
  sharing the reviewed byte/text/Serde admission implementation. Protocol remains
  on registry Types until a deliberately qualified cohort migration.

### 🧪 Testing

- Qualify an independent native/Wasm consumer with literal digest vectors,
  public Principal/Account crossings, published text/Serde comparisons and
  feature-specific dependency graph checks.

- Add independent literal identity and error fixtures, binary/human-readable Serde
  tokens, byte bounds, zero tails and length-first ordering checks.
- Record the narrowly retained Principal module's archive/VCS/licence provenance
  and remaining requirements in [qualification](../../docs/qualification/types-principal.md).
