#!/usr/bin/env bash
# Keep the setup entrypoint usable before the CI toolchain is installed.
exec perl "$(dirname "$0")/setup-step.pl" "$@"
