# PocketIC Rust client maintenance provenance

The maintainer authorized owning the Rust client while retaining the simulator
server and IC execution engine as external inputs. This first import preserves
method signatures, server HTTP/JSON bytes, Candid management payloads and current
operation behavior. A fallible API and provisioning-policy cut are separate work.

Source: `dfinity/ic`, commit `e9e42a6dd74acb9049017863e944cd5799be5b2b`,
`packages/pocket-ic`, published crate `pocket-ic 16.1.0`.
The original registry archive SHA-256 is
`85135917c845e8fc132f5eca3a0a8cbab696477a5fee3c2dda168a0797f81e1c`.
Retain DFINITY copyrights and the accompanying Apache-2.0 LICENSE.

The native-only implementation package is `ic-platform-pocket-ic`, selected only
through the existing facade feature and released with the Platform group.
Its Apache-2.0 license is an explicit exception to the workspace MIT default.

The retained implementation is `src/lib.rs`, `src/nonblocking.rs` and the two
`src/common/` files. Namespace/import paths and documentation/lint plumbing are
adapted to the maintained owner. Monorepo-dependent catalog comparisons are
replaced with source fingerprints and focused consumer/protocol qualification.
Numeric/version coverage and the original portable Windows-path tests remain;
installed-server qualification exercises Testkit. Upstream examples
are retained as documentation without implicitly executing server downloads.
No cryptographic implementation or protocol encoding body is rewritten.

Upstream review on 2026-11-01 found one later client change:
`bfddc1ae31af46004e35b7c7ac4433979013408a`, a hex-encoding clone optimization.
The initial import deliberately keeps the released 16.1.0 source and records this
review rather than mixing an unreleased performance change into ownership.
This observation is not a guarantee that the upstream repository has no advisory.

IC Platform owns maintenance of this client and its exposed native type identities.
IC Testkit continues to own its explicit server preparation, pin, compatibility,
process lifecycle and caller retry policy. Existing client convenience behavior is
retained in this signature-preserving import; it is not a new Testkit setup path.
The separately pinned native Profiler provider has its own patched server/client
protocol and is not automatically changed by this stock-client transfer.

Source inclusion alone does not establish offline build or simulator recovery.
Retain required server binaries, dependencies, compilers and tools independently.
