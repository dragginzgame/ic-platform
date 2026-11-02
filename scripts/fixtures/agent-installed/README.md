# Installed native Agent fixture

This consumer exercises the upstream `ic-agent` facade through a Testkit-admitted
PocketIC gateway: verified queries, update-and-wait, read-state and wrong-root
refusal with Ed25519, delegated, P-256 and secp256k1 identities. PocketIC lifecycle
calls use the separately maintained fallible client.

The public test PEM keys are copied unchanged from Agent 0.49.3 fixtures at
dfinity/agent-rs commit 0857d7dc6a47edbcc754ee41cfc66994ae38b254. Their Apache-2.0
license is retained in LICENSE-APACHE. They are test fixtures, not operational keys.

Run `scripts/ci/check-agent-installed.sh` with a prepared `POCKET_IC_BIN` or
`IC_TESTKIT_POCKET_IC_URL`. Local server startup requires loopback access.
