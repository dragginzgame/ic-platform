#!/usr/bin/env bash
# The group wrapper owns selection; verifier Make must not opt into a second recipe.
set -euo pipefail
unset CARGO_PUBLISH_PACKAGES
exec make "$@"
