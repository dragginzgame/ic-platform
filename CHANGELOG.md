# Changelog

Each Rust release group owns its version and ledger:

| Group | Packages | Release notes |
| --- | --- | --- |
| platform | ic-platform | [Platform](releases/platform/CHANGELOG.md) |
| runtime | ic-platform-runtime, ic-platform-macros, ic-platform-runtime-guard | [Runtime](releases/runtime/CHANGELOG.md) |
| types | ic-platform-types | [Types](releases/types/CHANGELOG.md) |

[Earlier coordinated releases](releases/history.md) retain their original scope.
Release and publication commands require RELEASE_GROUP=platform, runtime or types.
The browser npm package retains its [separate package](packages/browser/README.md).

The retired [PocketIC ledger](releases/pocket-ic/CHANGELOG.md) preserves published
history; the client now belongs to Testkit’s release group.

## Platform 0.21.0

### ⚠️ Breaking

- Move the maintained PocketIC client into Testkit and retire Platform's client
  package, facade feature and release group
  ([#139](https://github.com/dragginzgame/ic-platform/issues/139),
  [Tooling #324](https://github.com/dragginzgame/ic-tooling/issues/324)).
  See the [Platform release notes](releases/platform/CHANGELOG.md).
