# Latest Testkit and qualification server ownership

Ordinary qualification uses compatible requirements for Host Process, Testkit,
registry Types and WAT; `qualification/Cargo.lock` records the qualified versions
and archive checksums. Checks run locked and do not implicitly upgrade them.
Explicit frozen comparisons, including `published-platform` and historical
consumer fixtures, retain their selected exact versions. The pinned PocketIC
binary admission below is independent of these Cargo requirements.

Qualification selects published Testkit 0.107.4 (archive SHA-256
`8629db43fa7a3e3655184c6173a5ccdecf67a87d29f0c929228bec7f8a1c392b`).
Its upstream PocketIC 16.1.0 client replaces the removed `pic` helpers. Fixtures
explicitly connect to `POCKET_IC_SERVER_URL`, use a 30-second request deadline,
and delete their instances after successful execution. Auth's pinned fixture has
a retained ordinary `qualification/auth-runtime/testkit.patch`; no sibling is edited.

The private unpublished `platform-qualification-server` 0.1.0 binary supervises
one server and one qualification command. Native process ownership is delegated
to published Host Process 0.80.3 (archive SHA-256
`2b473c407b5d0e39597559bb9b506637849c6e7ad540ef45ab3b566b97244f82`),
which owns group signalling, unreaped leader custody and reaping. No process
implementation or unsafe code is copied. Command failure, startup refusal and
normal completion all close the server; cleanup errors fail qualification.
The 30-second readiness deadline remains. A 600-second server hard TTL and
570-second command deadline allow the actual Auth source build before installation.
The outer process supervisor has a 610-second bound. These are qualification
limits, not Runtime scheduling or application retry policies. Forced supervisor
termination has the server hard TTL as a fallback, not a successful cleanup claim.

`qualification/server.mjs` admits the prepared archive and executable before
running the supervisor. It checks the pinned upstream gzip SHA-256, bounded
archive/decompression sizes, exact decompressed executable digest and version.
Symlink asset files and directories are refused. Setup prepares a private sibling
directory and publishes the completed bundle; existing valid bundles are reused,
never silently replaced. It does not claim protection against a concurrent
malicious writer with access to the checkout. Test execution never downloads.

Only the three official 16.1.0 artifact digests/platform labels were retained from
published Testkit 0.106.0's `pic/server_assets` manifest, whose original license
notice remains in that supplier. No lifecycle, artifact or process implementation
was copied. The data identifies DFINITY PocketIC release artifacts, not executable
Rust. Native Linux observations do not establish macOS execution qualification.

The current published Platform 0.34.11 verifier also independently checks installed
Runtime certificates. Candidate 0.34.11 and registry 0.34.11 receive the same original
certificate bytes with separately constructed identities and explicit root/time/
resource policies. The registry archive SHA-256 is
`6269645258c22c1cccbaaf62ec710b7a30fbbcfe879aff6a53ba53b8526c9c84`.

This selection passed `make qualify-runtime-installed` on Linux on 2026-11-17,
including supervisor and asset checks, strict native/Wasm Clippy, installed
certificate observations, rollback/cleanup and stopped-upgrade scenarios, and
the Runtime singleton dependency graph check.
