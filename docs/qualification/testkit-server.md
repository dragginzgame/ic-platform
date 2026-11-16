# Latest Testkit and qualification server ownership

Qualification selects published Testkit 0.107.1 (archive SHA-256
`a4fe4ea7882118e731ef1ebe8c668fc5dfa5252c1c972b1de68bc444868fae42`).
Its upstream PocketIC 16.1.0 client replaces the removed `pic` helpers. Fixtures
explicitly connect to `POCKET_IC_SERVER_URL`, use a 30-second request deadline,
and delete their instances after successful execution. Auth's pinned fixture has
a retained ordinary `qualification/auth-runtime/testkit.patch`; no sibling is edited.

The private unpublished `platform-qualification-server` 0.1.0 binary supervises
one server and one qualification command. Native process ownership is delegated
to published Host Process 0.80.2 (archive SHA-256
`d4189c2a9262d34336c59ca7b22851cab1c629cd9afa98cd28a878e124d2e6d1`),
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

The current published Platform 0.34.4 verifier also independently checks installed
Runtime certificates. Candidate 0.34.4 and registry 0.34.4 receive the same original
certificate bytes with separately constructed identities and explicit root/time/
resource policies. Legacy 0.33.7 remains named `legacy-platform` solely for retained
codec comparisons; it is not claimed as the latest Platform or used to implement
candidate production capabilities.
