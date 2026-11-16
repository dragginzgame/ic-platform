# Changelogs

Each product release group owns its version and changelog.

| Group | Packages | Changelog |
| --- | --- | --- |
| Protocol | `ic-platform-protocol` | [Protocol changelog](releases/protocol/CHANGELOG.md) |
| Types | `ic-platform-types` | [Types changelog](releases/types/CHANGELOG.md) |
| Runtime | `ic-platform-runtime` and support packages | [Runtime changelog](releases/runtime/CHANGELOG.md) |
| Platform | `ic-platform` | [Platform changelog](releases/platform/CHANGELOG.md) · [subnet-time #1](https://github.com/dragginzgame/ic-platform-new/issues/1) |

Runtime's guard and macros share its ledger. Browser will gain a separate ledger
when its implementation is introduced. Qualification
packages are unpublished and outside product release groups.

The [group policy](ci/cargo-release-groups.json) lists implemented groups only.
These notes do not imply a release or registry publication. See
[release-group setup](docs/development.md#release-group-ownership) for the boundary
between this layout and future release execution.
