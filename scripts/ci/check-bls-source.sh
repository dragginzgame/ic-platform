#!/usr/bin/env bash
set -euo pipefail
# Reconstruct the original source, admitting only the exact reviewed #127 fix.
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
[[ $# -le 1 ]] || exit 2
source_root="${1:-$root/crates/ic-platform/src/bls/upstream}"
scratch="$(mktemp -d "${TMPDIR:-/tmp}/platform-bls-source.XXXXXX")"
trap 'rm -rf "$scratch"' EXIT
perl -0777 -e '
    use strict; use warnings;
    my $s = <>;
    my $entry = q!    pub fn verify(&self, message: &[u8], signature: &Signature) -> Result<(), ()> {!;
    my $fix = "\n        if self.pk.is_identity().into() {\n            return Err(());\n        }\n";
    ($s =~ s/\Q$entry$fix\E/$entry/) == 1 or die "identity-key rejection must remain at verifier entry\n";
    ($s =~ s{use super::curve as bls12_381; // Platform: maintained curve implementation\.\n\n}{}) == 1 or die "curve import changed\n";
    my $expect = q{#![expect(clippy::result_unit_err, reason = "Preserve the upstream unit-error API.")]};
    my $new_cache = "static G2PREPARED_NEG_G: std::sync::LazyLock<bls12_381::G2Prepared> =\n    std::sync::LazyLock::new(|| G2Affine::generator().neg().into());";
    my $old_cache = "lazy_static::lazy_static! {\n    static ref G2PREPARED_NEG_G : bls12_381::G2Prepared = G2Affine::generator().neg().into();\n}";
    my $tests = "\n#[cfg(test)]\n#[rustfmt::skip]\nmod tests; // Platform: original upstream tests.\n";
    ($s =~ s{\A// Platform: std is supplied by the facade\.\n}{#![no_std]\n}) == 1 or die "std admission changed\n";
    ($s =~ s/\Q$expect\E/#![allow(clippy::result_unit_err)]/) == 1 or die "lint admission changed\n";
    ($s =~ s{// Platform: alloc implementation selected\.}{#[cfg(feature = "alloc")]}g) == 5 or die "alloc selection changed\n";
    ($s =~ s{// Platform: retain the alloc Debug implementation\.}{#[cfg_attr(not(feature = "alloc"), derive(Debug))]}g) == 2 or die "Debug selection changed\n";
    ($s =~ s{\Q#[cfg(any())] // Platform: preserve the unselected upstream no-alloc alternative.\E}{#[cfg(not(feature = "alloc"))]}) == 1 or die "no-alloc admission changed\n";
    ($s =~ s{\Q#[cfg(test)] // Platform: upstream RNG helper is test-only.\E}{#[cfg(feature = "rand")]}) == 1 or die "RNG admission changed\n";
    ($s =~ s/\Q$new_cache\E/$old_cache/) == 1 or die "cache admission changed\n";
    ($s =~ s/\Q$tests\E\z//) == 1 or die "test module admission changed\n";
    print $s;
' "$source_root/mod.rs" > "$scratch/source"
perl -0777 -e '
    use strict; use warnings;
    use Digest::SHA qw(sha256_hex);
    my $s = <>;
    my $added;
    ($s =~ s{(#\[test\]\nfn reject_invalid_key_and_signature\(\) \{.*?^\}\n\n)}{$added = $1; ""}mse) == 1 or die "missing identity-key vector\n";
    sha256_hex($added) eq "f3d7262fd44bf484c7e4cda0b64c53714f4819764407302391e92f50f3177873" or die "upstream identity-key vector changed\n";
    ($s =~ s/^use super::\*;/use ic_verify_bls_signature::*;/m) == 1 or die "test import changed\n";
    ($s =~ s{// Platform: upstream RNG test enabled in test builds\.}{#[cfg(feature = "rand")]}g) == 2 or die "random tests changed\n";
    ($s =~ s/rng\.r#gen::</rng.gen::</) == 1 or die "edition admission changed\n";
    print $s;
' "$source_root/tests/mod.rs" > "$scratch/tests"
bash "$root/scripts/ci/verify-file-checksum.sh" sha256 \
    6e6e19c71bafc42c29d4366424f02c65ee06494a00f6e74738a5f87f682a3c42 "$scratch/source"
bash "$root/scripts/ci/verify-file-checksum.sh" sha256 \
    53d379ae9a10ce09bd12194baec5f4c6201192b1c260e15f3fb0403a448b4165 "$scratch/tests"
bash "$root/scripts/ci/verify-file-checksum.sh" sha256 \
    167e55afedec201ea30f4980c5ef0c8204dbbe1eb8196f75d43897b4ad8f2bf3 "$source_root/Cargo.toml.original"
bash "$root/scripts/ci/verify-file-checksum.sh" sha256 \
    cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30 "$source_root/LICENSE-APACHE"
echo 'BLS source matches the original import plus the exact upstream #127 identity-key rejection and vector.'
