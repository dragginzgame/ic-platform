#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/platform-runtime-source-test.XXXXXX")"
trap 'rm -rf "$fixture"' EXIT
for mutation in cleanup protected license; do
    cp -R "$root/crates/ic-platform-runtime/src/cdk/upstream" "$fixture/$mutation"
    case "$mutation" in
        cleanup) perl -i -pe 's/reject_message: "cleanup"/reject_message: "changed cleanup"/' "$fixture/$mutation/call/mod.rs" ;;
        protected) perl -i -pe 's/if !this.completed/if this.completed/' "$fixture/$mutation/futures/mod.rs" ;;
        license) printf '\nchanged\n' >> "$fixture/$mutation/LICENSE-APACHE" ;;
    esac
    if bash "$root/scripts/ci/check-owned-runtime-source.sh" "$fixture/$mutation" > "$fixture/log" 2>&1; then
        echo "Unexpectedly admitted runtime source mutation: $mutation" >&2; exit 1
    fi
done
printf 'Cleanup, protected-spawn and license mutation refusals passed.\n'
for module in ic0/upstream executor/runtime; do
    for file in mod.rs LICENSE-APACHE; do
        destination="$fixture/embedded-${module//\//-}-${file//./-}"
        mkdir -p "$destination/ic0" "$destination/executor"
        cp -R "$root/crates/ic-platform-runtime/src/ic0/upstream" "$destination/ic0/"
        cp -R "$root/crates/ic-platform-runtime/src/executor/runtime" "$destination/executor/"
        printf '\nmutated source\n' >> "$destination/$module/$file"
        if bash "$root/scripts/ci/check-embedded-runtime-source.sh" "$destination" > "$fixture/log" 2>&1; then
            echo "Unexpectedly admitted embedded runtime mutation: $module/$file" >&2; exit 1
        fi
    done
done
for part in creation verification; do
    for file in mod.rs LICENSE; do
        destination="$fixture/signature-$part-${file//./-}"
        cp -R "$root/crates/ic-platform/src/signature_upstream" "$destination"
        printf '\nmutated source\n' >> "$destination/$part/$file"
        if bash "$root/scripts/ci/check-signature-source.sh" "$destination" > "$fixture/log" 2>&1; then
            echo "Unexpectedly admitted signature mutation: $part/$file" >&2; exit 1
        fi
    done
done
printf 'Signature source, vector and license mutation refusals passed.\n'
for file in mod.rs tests/mod.rs LICENSE-APACHE; do
    destination="$fixture/bls-${file//\//-}"
    cp -R "$root/crates/ic-platform/src/bls/upstream" "$destination"
    printf '\nmutated source\n' >> "$destination/$file"
    if bash "$root/scripts/ci/check-bls-source.sh" "$destination" > "$fixture/log" 2>&1; then
        echo "Unexpectedly admitted BLS source/vector/license mutation: $file" >&2; exit 1
    fi
done
printf 'BLS source, vector and license mutation refusals passed.\n'
for file in scalar.rs hash_to_curve/map_g1.rs tests/mod.rs tests/g1_compressed_valid_test_vectors.dat LICENSE-MIT LICENSE-APACHE COPYRIGHT; do
    destination="$fixture/curve-${file//\//-}"
    cp -R "$root/crates/ic-platform/src/bls/curve" "$destination"
    printf '\nmutated source\n' >> "$destination/$file"
    if bash "$root/scripts/ci/check-bls-curve-source.sh" "$destination" > "$fixture/log" 2>&1; then
        echo "Unexpectedly admitted BLS curve source/vector/license mutation: $file" >&2; exit 1
    fi
done
destination="$fixture/curve-selection"
cp -R "$root/crates/ic-platform/src/bls/curve" "$destination"
perl -i -pe 's/any\(\)/all()/g' "$destination/scalar.rs"
if bash "$root/scripts/ci/check-bls-curve-source.sh" "$destination" > "$fixture/log" 2>&1; then
    echo 'Unexpectedly admitted changed BLS curve feature selection' >&2; exit 1
fi
printf 'BLS curve source, vectors, license and feature-selection mutation refusals passed.\n'
for file in mod.rs certificate.rs hash_tree/mod.rs hash_tree/tests.rs rb_tree/mod.rs rb_tree/tests.rs LICENSE-APACHE; do
    destination="$fixture/certification-${file//\//-}"
    cp -R "$root/crates/ic-platform/src/certification/upstream" "$destination"
    printf '\nmutated source\n' >> "$destination/$file"
    if bash "$root/scripts/ci/check-certification-source.sh" "$destination" > "$fixture/log" 2>&1; then
        echo "Unexpectedly admitted certification source/vector/license mutation: $file" >&2; exit 1
    fi
done
printf 'Certification source, vector and license mutation refusals passed.\n'
for mutation in cache noalloc rng identity-missing identity-changed identity-moved identity-vector; do
    destination="$fixture/bls-admission-$mutation"
    cp -R "$root/crates/ic-platform/src/bls/upstream" "$destination"
    case "$mutation" in
        cache) perl -i -pe 's/LazyLock::new/OnceLock::new/' "$destination/mod.rs" ;;
        noalloc) perl -i -pe 's/cfg\(any\(\)\)/cfg(all())/' "$destination/mod.rs" ;;
        rng) perl -i -pe 's/\#\[cfg\(test\)\] \/\/ Platform: upstream RNG/#[cfg(all())] \/\/ Platform: upstream RNG/' "$destination/mod.rs" ;;
        identity-missing) perl -0777 -i -pe 's/        if self\.pk\.is_identity\(\)\.into\(\) \{\n            return Err\(\(\)\);\n        \}\n\n//' "$destination/mod.rs" ;;
        identity-changed) perl -i -pe 's/self\.pk\.is_identity\(\)/self.pk.is_torsion_free()/' "$destination/mod.rs" ;;
        identity-moved) perl -0777 -i -pe 's/(        if self\.pk\.is_identity\(\)\.into\(\) \{\n            return Err\(\(\)\);\n        \}\n\n)(        let msg = hash_to_g1\(message\);\n)/$2$1/' "$destination/mod.rs" ;;
        identity-vector) perl -i -pe 's/fn reject_invalid_key_and_signature/fn removed_identity_vector/' "$destination/tests/mod.rs" ;;
    esac
    if bash "$root/scripts/ci/check-bls-source.sh" "$destination" > "$fixture/log" 2>&1; then
        echo "Unexpectedly admitted BLS scaffolding mutation: $mutation" >&2; exit 1
    fi
done
printf 'Changed BLS cache, verification-path and RNG admissions are refused.\n'
for mutation in missing duplicate moved; do
    destination="$fixture/signature-admission-$mutation"
    cp -R "$root/crates/ic-platform/src/signature_upstream" "$destination"
    case "$mutation" in
        missing) perl -i -ne 'print unless /Platform: checked DER admission\./' "$destination/creation/mod.rs" ;;
        duplicate) perl -i -pe 'print if /Platform: checked DER admission\./' "$destination/creation/mod.rs" ;;
        moved) perl -0777 -i -pe 's/(    super::admission::canister_sig_der\(pk_der\)\?; \/\/ Platform: checked DER admission\.\n)(    let oid_part = [^\n]+\n)/$2$1/' "$destination/creation/mod.rs" ;;
    esac
    if bash "$root/scripts/ci/check-signature-source.sh" "$destination" > "$fixture/log" 2>&1; then
        echo "Unexpectedly admitted changed DER admission: $mutation" >&2; exit 1
    fi
done
printf 'Missing, duplicate and moved DER admission calls are refused.\n'
for file in mod.rs LICENSE-APACHE; do
    destination="$fixture/management-${file//./-}"
    cp -R "$root/crates/ic-platform/src/management_canister_types/maintained" "$destination"
    printf '\nmutated\n' >> "$destination/$file"
    if bash "$root/scripts/ci/check-management-source.sh" "$destination" > "$fixture/log" 2>&1; then
        echo "Unexpectedly admitted management source/license mutation: $file" >&2; exit 1
    fi
done
printf 'Management DTO source and license mutation refusals passed.\n'
