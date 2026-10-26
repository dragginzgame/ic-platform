# Current handoff

## Browser SDK Core boundary

This batch starts at released Platform 0.6.16 (`0b84c2e`). Pending root notes use
0.6.17; Rust package versions remain 0.6.16. The independent npm candidate is
`@dragginzgame/ic-platform` 0.1.0 under `packages/browser`, still unpublished.
No commit, push, release or upload was performed. Siblings remain read-only.
Shared Tooling is 0.26.18 (`37652c0`), refreshed through its canonical exporter.
Normal releases start fresh; exact recovery uses `release-resume VERSION=X.Y.Z`.
Common external IC dependency guards supplement Platform's stricter local policy.
Snapshot, offline tool admission/reporting, formatting, release wiring and focused
release/installer/guard fixtures pass on Linux after full Clippy. Evidence is at
`target/shared-adoption.5kYsS9` and `/tmp/platform-shared-*.log`; macOS is not rerun.
Full `check-pins` awaits Git-tracking the pending browser manifest/lock. Its pin
data checks pass; no staging, release, tool installation or publication was performed.

The candidate now selects exactly SDK Core **6.1.1** as an ordinary dependency
for Blob's approved local 0.48 graph. This supersedes the unpublished SDK-5
candidate. Explicit ESM subpaths cover Principal, Ed25519/secp256k1 identities,
Agent/request APIs, certificates and required external Candid. No blanket root
export, SDK fork, new crypto implementation, transport or Wire generator exists.

Owned adapters provide bounded canonical Principal text, checked identity JSON
import with caller-selected budgets, explicit trust/Agent options and raw-query
rejections. Buffer views are copied without aliasing; invalid query byte shapes
are refused before dispatch. SDK-6 rejection context preserves effective canister
or subnet routing, including the SDK's deprecated effective-canister input.
Automatic root discovery is disabled by the options helper; consumers explicitly
select retries, query verification and construction-time synchronization.

Wire retains owned application codecs/generation. Consumers retain trust roots,
authorization, budgets, sessions, storage, cancellation and effect reconciliation.
Identity crypto, request signing/hashing, certificate verification and external
Candid remain upstream. No Rust public surface or dependency selection changed.
The [package contract](../../packages/browser/README.md),
[migration guide](../migration.md#blob-048-and-sdk-6) and
[inventory](../ic-inventory.md) describe supported imports and ownership.

## Blob 0.48 qualification

Read-only review used Blob `c4fe55dd` with its dirty pending 0.48 work preserved.
All current browser SDK imports fit the existing exports; no new exports were
needed. Disposable copies adopt checked restoration with Blob's 512-byte budget,
canonical text and explicit Agent options, retaining IntentAgent/guarded fetch,
zero retries, exact signed bytes and durable uncertainty/cancellation fences.

Full root `make clippy` passed before longer tests. `make check-browser-runtime`
passes fifteen adapter/graph tests, packed-only strict Node/TypeScript consumers
with no direct SDK declaration, export-map privacy, Principal bundle isolation, and actual
Chromium page/module-worker checks. Provider types compile with Caffeine 1.1.2
and Auth 10.0.2 on one SDK 6.1.1 graph. Latest packed browser evidence:
`target/browser-consumer.qnGGb4`; logs: `/tmp/platform-browser-followup-*.log`.
Tool/packed checks reject duplicate/aliased locks, installed SDK drift and shadow
Agent copies; regular and packed license copies must match the root MIT owner.

Seven real Blob browser entry bundles use one SDK root. Copied Blob suites pass:
35 frozen-publication cases, 58 worker cases, 22 native browser/profile cases,
17 Chromium signer/bootstrap cases and IndexedDB persistence/restart checks.
The Caffeine source hashes and reviewed integration patch are preserved.
Source hashes, bundle maps, reports and registry readback:
`target/blob048.4V8Lib/final`; copied sources: `target/browser-consumer.uLItSc/blob048`.
These are local packed-artifact/source-migration proofs. Installed IC/live-provider,
billing, real login, full release CI and native macOS are not qualified here.
Blob's prior SDK-6 installed tests remain consumer-owned evidence, not proof of
published Platform adoption. The earlier SDK-5/Canic source-copy evidence remains
at `target/browser-consumer.IIPjBv/adoption` and does not qualify this SDK-6 change.
The reviewed 16-file source migration patch is retained at
`target/browser-followup.bbsRby/blob048-source-migration.patch`; read-only
`git apply --check` passes against Blob. Manifests/supply remain separate.

Contributor tools are Node 24.21.0/npm 12.2.0 and prepared Linux Chromium. Other
engines/Node versions remain unqualified. No size or performance gain is claimed.
Browser CI now selects Ubuntu and native macOS 15 Intel/Apple Silicon, with host
identity admission. The new macOS jobs have not run; Linux is the local evidence.
The source-publish hook and real offline no-upload npm publication preview pass:
`/tmp/platform-browser-followup-publish.log`, packed proof `browser-consumer.KZtLUw`.
The preview reports login required; it does not prove registry ownership.
Rust `make publish` remains crates.io-only.

## Supply and adoption

[Platform #57](https://github.com/dragginzgame/ic-platform/issues/57) owns review,
integration and npm supply. Public npm readback still returns HTTP 404 on
2026-10-26. Maintainer organization/package permission and publication remain
outstanding; local archives are qualification artifacts, not supported supply.
[Blob #103](https://github.com/dragginzgame/ic-blob-storage/issues/103) owns adopting
the published artifact, direct SDK declaration/import refusal and final consumer
acceptance. Existing SDK peers stay until supply. Caffeine 1.1.2 declares SDK 5:
retain Blob's application-owned SDK-6 override and reviewed patch. On removing the
direct SDK declaration, replace `$@icp-sdk/core` with exact `6.1.1` in the override.
Platform's development override is not inherited by applications.

[Canic #637](https://github.com/dragginzgame/canic/issues/637) needs a coordinated
Core/Auth migration: its Core 5.4/Auth 8 declarations remain incompatible with a
single SDK-6 graph. Auth 10.0.2 is a producer type fixture, not Canic login/session
acceptance. Preserve Canic #613's owner/build deferral. Compiler retirement stays
on Platform #38 after committed consumer adoption. Advisory-index #56 is unchanged.

## Rust qualification and advisory boundary

The incoming Wire 0.7.12 lock update is preserved. Its derive selection advanced
to 0.7.14 during tooling validation and was retained. The two host fixtures use
the explicit acknowledgement/diagnostics API. Focused Rust 1.91 and generated
Principal/Account byte proofs passed with the prior 0.7.12 derive selection;
Full Clippy passes the current graph; those earlier runtime proofs remain separate.
Detailed released runtime/source evidence remains at `0b84c2e:docs/status/current.md`.
Executor =2.0.0, ic0 =1.2.0, external bytes and crypto/runtime guards are unchanged.

The [#50](https://github.com/dragginzgame/ic-platform/issues/50) recheck uses RustSec
`21d909e` and index records at `target/issue50-20261026`. The workspace still flags
crypto-bigint 0.5.5 through ECDSA/Agent/public derivation; the separate runtime lock
reports no vulnerabilities with yanked checks skipped. Defaults/executor/runtime
normal/build graphs exclude it. No compatible supplier or signing exemption is
established. A complete correction needs a reviewed three-path dependency/type
migration. That investigation changed no crypto bodies, suppressions or dependency
selections; the separate Wire lock activity above does not correct this advisory.
