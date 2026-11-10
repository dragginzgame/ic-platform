# Rejection catalogue implementation record

On 2026-11-10 a separate implementer context authored Runtime's rejection
catalogue and tests from the reviewed [contract](reject-code-contract.md).
The integrating reviewer supplied the task and retained integration ownership.

## Inputs and separation

The implementer was permitted to read, and read, only these existing repository
files, relative to `/home/adam/projects/ic-platform`:

- `AGENTS.md`
- `TOKO-TOOLING.md`
- `docs/design/reject-code-contract.md`
- `docs/principles/rust-code-hygiene.md`

The reviewer also authorized a policy-only read of `rules/rust-hygiene.md`;
that path did not exist and supplied no content. No broad file search followed.
The reviewer clarified that `crate::cdk::call::RejectCode` is the public test
path and that its conversion error's `.0` payload may be accessed by inference;
no public call-module re-export of `UnknownRejectCode` was required.

The implementer received no previous Rust implementation or tests in its task
context. It did not read the previous module, previous tests, Git history or
diffs, source-inventory reports, upstream implementations, or
`docs/status/current.md`. The reviewer's explicit task boundary replaced the
usual handoff and pre-edit source reads for this narrow implementation. The
protocol link in the supplied contract was not independently opened by the
implementer. No other agent was delegated work.

Process inspection was used to check for Cargo/rustc activity before editing;
no process with either exact command name was present. This inspection was
operational evidence, not implementation input.

## Authored files

The implementer wrote complete new contents without first reading their
previous contents:

- `crates/ic-platform-runtime/src/cdk/reject_code/mod.rs`
- `crates/ic-platform-runtime/src/cdk/reject_code/tests.rs`
- `docs/design/reject-code-implementation.md`

The module uses explicit protocol discriminants, a checked `u64` match,
formatting from static labels, and Serde derives. It adds no unsafe code,
dependency, layout representation attribute, public helper or retry policy.
The tests were written from the contract's table and requirements. They cover
all six numeric casts, conversions, display labels and Debug names; exact
unsupported input preservation; Serde unit variants and declaration-index
deserialization; unknown variant refusal; and the public call path.

The implementer did not change manifests or license files. Removal of the old
`reject_code/tests/mod.rs`, integration documentation and broader qualification
remain with the integrating reviewer.

## Validation and limitations

The following formatting command completed successfully with no output:

```text
rustfmt +1.99.1 --edition 2024 --config-path /dev/null --config skip_children=true crates/ic-platform-runtime/src/cdk/reject_code/mod.rs crates/ic-platform-runtime/src/cdk/reject_code/tests.rs
```

The same command with `--check` also completed successfully with no output.
The explicit empty configuration and `skip_children` confined formatting to the
two authored Rust files, without traversing old test modules. The implementer
ran no Cargo command, Clippy, compilation or tests. The integrating reviewer
owns those checks and their results; this record does not report them as passed.

This is an account of the inputs and actions of one separated implementation
context. It is not legal clearance, a claim about model training exposure, or
mathematical proof of independent authorship. Behavioral test success alone
would not establish those conclusions.
