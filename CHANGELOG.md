# Changelogs

Each product release group owns its version and changelog.

| Group | Packages | Changelog |
| --- | --- | --- |
| Protocol | `ic-platform-protocol` | [Protocol changelog](releases/protocol/CHANGELOG.md) |
| Types | `ic-platform-types` | [Types changelog](releases/types/CHANGELOG.md) |

Runtime (including its guard and macros), Platform and Browser will gain
separate ledgers as their implementation packages are introduced. Qualification
packages are unpublished and outside product release groups.

The [group policy](ci/cargo-release-groups.json) lists implemented groups only.
These notes do not imply a release or registry publication. See
[release-group setup](docs/development.md#release-group-ownership) for the boundary
between this layout and future release execution.
