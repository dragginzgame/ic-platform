#!/usr/bin/env bash
# Cargo releases its target lock before returning. Keep this inherited lock
# through the copy so another cooperative proof cannot replace the artifact.
# Perl flock is available on both supported Unix hosts; the subshell owns fd 9.
retain_build_artifact() (
    local artifact_cache="$1" artifact_relative="$2" artifact_destination="$3"
    shift 3
    mkdir -p "$artifact_cache"
    exec 9>>"$artifact_cache/.platform-artifact.lock"
    perl -MFcntl=:flock -e 'open my $lock, "<&=9" or die "$ARGV[0]: $!"; flock($lock, LOCK_EX) or die "$ARGV[0]: $!"' "$artifact_cache/.platform-artifact.lock"
    export CARGO_TARGET_DIR="$artifact_cache"
    "$@" || exit
    cp "$artifact_cache/$artifact_relative" "$artifact_destination"
)
