# Installed native Agent fixture

This consumer exercises the upstream `ic-agent` facade through a Testkit-admitted
PocketIC gateway: verified queries, update-and-wait, read-state and wrong-root
refusal with Ed25519, delegated, P-256 and secp256k1 identities. PocketIC lifecycle
calls use the separately maintained fallible client.

The public test PEM keys were generated for this repository on 2026-11-10 using
OpenSSL 3.0.13. They replace the previously copied Agent fixtures; no key from
that fixture source remains. Cryptographic key generation and parsing stay with
OpenSSL and the upstream Agent packages. These deliberately published test keys
must never be used as operational identities.

Generation commands (each invocation creates new random keys):

```sh
openssl ecparam -name prime256v1 -genkey -noout -out prime256v1.pem
openssl ecparam -name secp256k1 -genkey -noout -out secp256k1.pem
```

Both keys passed `openssl ec -in FILE -check -noout`. Installed qualification
uses the files directly; it does not depend on a specific principal or signature.
Generating different valid keys requires rerunning that identity proof.

Run `scripts/ci/check-agent-installed.sh` with a prepared `POCKET_IC_BIN` or
`IC_TESTKIT_POCKET_IC_URL`. Local server startup requires loopback access.
