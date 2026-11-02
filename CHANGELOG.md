# Changelog

Each Rust release group owns its version and ledger:

| Group | Packages | Release notes |
| --- | --- | --- |
| platform | ic-platform | [Platform](releases/platform/CHANGELOG.md) |
| pocket-ic | ic-platform-pocket-ic | [PocketIC](releases/pocket-ic/CHANGELOG.md) |
| runtime | ic-platform-runtime, ic-platform-macros, ic-platform-runtime-guard | [Runtime](releases/runtime/CHANGELOG.md) |
| types | ic-platform-types | [Types](releases/types/CHANGELOG.md) |

[Earlier coordinated releases](releases/history.md) retain their original scope.
Release and publication commands require RELEASE_GROUP=platform, pocket-ic, runtime or types.
The browser npm package retains its [separate package](packages/browser/README.md).
