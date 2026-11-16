# Types changelog

Release group: `types`. Covered package: `ic-platform-types`.
Return to the [group index](../../CHANGELOG.md).

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
