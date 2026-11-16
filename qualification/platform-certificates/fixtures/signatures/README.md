# Observed canister signatures

Captured locally on 2026-11-16 by `make qualify-canister-signature-observed`,
using PocketIC 16.1.0 through ic-testkit 0.106.0. The independent
[`signature.wat`](../../signature.wat) probe admits a 32-byte initialization
argument as certified data and returns the replica's exact certificate bytes.
The native observer constructs the witness with upstream ic-certification 3.2.0,
installs its digest, then assembles the signature envelope with serde_cbor 0.11.2.
Neither the candidate's verifier nor its synthetic signing helpers create this
evidence. The certificate signatures and delegation come from the replica;
the witness, DER framing and outer envelope are qualification-owned data.
This is a controlled observation, not a capture from Auth or a mainnet signer.

The witness proves an empty `sig / SHA256(seed) / SHA256(message)` leaf, following
the [IC canister-signature contract](https://docs.internetcomputer.org/references/ic-interface-spec/).
The literal hash labels were obtained independently with Node's `crypto` SHA-256:

| Input (UTF-8, no trailing newline) | SHA-256 |
| --- | --- |
| `qualification-seed` | 47e52500acfa5a926e4f26f1b9fbb4551ed1a872e7f55e517523944a6aab47f6 |
| `ic-platform observed signature` | a988b1e93f816e575a9def847432d02d861cc862855487acf3a8303e1a373ace |

The root is explicitly obtained from this simulator instance, separately from
the envelopes. It is a public qualification key, never a product trust default.
The two `.der` issuer keys use the specified canister-signature OID, the
ten-byte issuer and the same 18-byte seed. No upstream source or fixtures were
copied. No cryptographic primitive is implemented by the probe.

| Profile | Issuer | Signing time (Unix ns) | Delegation time (Unix ns) |
| --- | --- | --- | --- |
| Direct | `xp3jw-ot777-77777-aaaaa-cai` | 1620328630000000005 | None |
| Delegated | `dllsh-pd777-77776-qaaaa-cai` | 1620328630000000008 | 1620328630000000008 |

The direct issuer is on root subnet
`hzklm-c4flk-lfdw4-v4532-s744u-jbwhv-6zh6w-be4il-apmtb-fosk7-wae`.
The delegated issuer is on application subnet
`sgier-p6fyz-ejpwm-jghgc-4ypgv-6ft53-5hns6-ojhae-wvlic-ifcah-eqe`.
These are simulator identities and times, not the wall clock of capture.
Offline replay selects separate, exact signing/delegation time windows.

| File | Bytes | SHA-256 |
| --- | --- | --- |
| direct.cbor | 538 | 3dd0531b03cb9c16dd1101412eb65c501ac0d94f3af43a247d3c504fb79c34ff |
| delegated.cbor | 1249 | ef287630e3d34ccc4624d139502bb3ad2f7958ca5a7fc1d56ba3e381af804c1d |
| direct.der | 48 | 8262c206a8b6d8cc21ba2b87b9e28ff7d66ef959c54c1c416b33de754dba292a |
| delegated.der | 48 | 2d546c93ff47b3857c06fa293aa364b6b7b4625bb203e5d032e0e8e8d64e2d11 |
| root.der | 133 | 968c6c2122d3577878be624c1dc37609286f2f9ad7ee2cdae5d6bd45f0a05e8c |

Capture used Rust 1.100.0 on Linux x86-64, wat 1.263.0, and the selected
qualification lockfile (SHA-256
`55819d54d7884a9c77fb6ed3f4515dfec91532ebfd0d6c8531305dbf88b62f0f`).
The server's admitted SHA-256 is
`b44e1eccd66e02328146b3e209e3403e3a3428a25d25555aca2f67500e5ae9db`.
Probe WAT SHA-256:
`64a73dd928330937c353556ee58e6f330a3ec1e73404f57b7f7678b18faa9f76`;
compiled probe SHA-256:
`e7fbc7ac8ab97372de4fc86efa6ed33368a80a7642bef42aaeaac1b87909db59`.

Fresh observations go to `target/platform-signatures/`, including the compiled
probe and context text. That command never replaces these frozen fixtures.
Replay consumes the retained bytes without a server or network connection;
recapture requires reviewing the new context and hashes before replacing them.

## Auth message observations

The observer now additionally installs one issuer per subnet for the exact
Auth message `0x1b || UTF8("ic-auth-portable-credential") || [0x42; 32]`.
Its 60-byte framing and SHA-256
`c055f2e59f0d7d8d29730b0d2b15d041ccc01b135786dde048f6dc1001b15ded`
were independently calculated with Node crypto. The seed is unchanged.
These are newly captured replica certificates, not edits to the earlier frames.

| File | Bytes | SHA-256 |
| --- | --- | --- |
| auth-direct.cbor | 576 | a6c531949883f87988dfc7a8c438d0dda7a426d764fee89f93a444c76218be74 |
| auth-delegated.cbor | 1287 | eea4144614e111818a2badb9d566d34dcf358cd9bb36c3ebed2a7cae6c5c02db |
| auth-direct.der | 48 | 50c0fd1de1d7f0174a09e9bdb2e1deebf0c05e1ccd174ce6c0925ae4641a7612 |
| auth-delegated.der | 48 | 669baa1dc991fc66d716c55f96af10e8016ce5c3a3f742e2a2e645f9a0410202 |

The direct issuer is `xi2pc-dl777-77777-aaaaq-cai` with signing time
`1620328630000000011`. The delegated issuer is `dmkut-c3777-77776-qaaaq-cai`
with signing/delegation time `1620328630000000014`. The explicit root, subnets,
server and probe Wasm are unchanged from the observation above.
The qualification lock SHA-256 at this capture is
`33581ad3b0e8f7610000c8c4cbbd47ba9834f16ed9064e99c1390812a1ff70cc`;
the observer module SHA-256 is
`db3e305a321caf72bd39011bdd69645f9f4475d34cd9f018ee1d6d2d64e1e4f3`.
Capture date, compiler and host match the first observation. The public consumer
uses fixed approved issuers and seed commitments; submitted keys do not select
the trusted policy. Full claims validation and actual Auth adoption are not
part of these fixtures.
