# Changelog

Each Rust release group owns its version and ledger:

| Group | Packages | Release notes |
| --- | --- | --- |
| platform | ic-platform | [Platform](releases/platform/CHANGELOG.md) |
| runtime | ic-platform-runtime, ic-platform-macros, ic-platform-runtime-guard | [Runtime](releases/runtime/CHANGELOG.md) |
| types | ic-platform-types | [Types](releases/types/CHANGELOG.md) |

[Earlier coordinated releases](releases/history.md) retain their original scope.
Release and publication commands require RELEASE_GROUP=platform, runtime or types.
The browser npm package retains its [separate package](packages/browser/README.md)
and [release notes](packages/browser/CHANGELOG.md).

The retired [PocketIC ledger](releases/pocket-ic/CHANGELOG.md) preserves published
history; the client now belongs to Testkit’s release group.
