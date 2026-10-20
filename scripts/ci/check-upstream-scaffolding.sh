#!/usr/bin/env bash
set -euo pipefail
# Check the packaging proposals without selecting them in the production graph.
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
command -v patch > /dev/null || { echo 'Source qualification requires the system patch utility' >&2; exit 1; }
mkdir -p target
attempt="$(mktemp -d "$root/target/upstream-scaffolding.XXXXXX")"
printf 'Scaffolding source evidence: %s\n' "$attempt"
jq="$root/.tools/host/bin/jq"
host="$(rustc -vV | sed -n 's/^host: //p')"
cargo metadata --all-features --format-version 1 --filter-platform "$host" --frozen --offline > "$attempt/metadata.json"
for selection in \
    'secp ic-secp256k1 0.3.0 secp256k1-optional-scaffolding'; do
    read -r directory package version proposal <<< "$selection"
    manifest="$("$jq" -er --arg package "$package" --arg version "$version" '
        [.packages[] | select(.name == $package)] as $p |
        if ($p | length) == 1 and $p[0].version == $version and
           $p[0].source == "registry+https://github.com/rust-lang/crates.io-index"
        then $p[0].manifest_path else error("expected exact registry source") end
    ' "$attempt/metadata.json")"
    original="${manifest%/*}"
    candidate="$attempt/$directory"
    cp -R "$original" "$candidate"
    # Git apply can skip these paths when the fixture is nested in this checkout.
    # Patch operates on the explicit copied directory without Git discovery.
    patch -p1 -F 0 -f -d "$candidate" < "$root/patches/$proposal.patch"
    # Refuse a silently skipped patch even if unchanged sources would compare.
    case "$directory" in
        creation) rg -q '^signature-map = \["dep:ic0"\]$' "$candidate/Cargo.toml" ;;
        secp) rg -q '^full = ' "$candidate/Cargo.toml" ;;
        verifier)
            "$root/.tools/host/bin/yq" -p toml -o json '.' "$candidate/Cargo.toml" |
                "$jq" -e '.dependencies["ic-canister-sig-creation"]["default-features"] == false' > /dev/null ;;
    esac
    cmp "$original/LICENSE" "$candidate/LICENSE"
    # Every source file must retain its original bytes after removing only the
    # supported packaging attributes and canonical Principal import adjustment.
    rg --files "$original/src" > "$attempt/$directory.sources"
    while IFS= read -r source; do
        relative="${source#"$original/"}"
        if [[ "$relative" == src/lib.rs && "$directory" != verifier ]]; then
            case "$directory" in
                creation)
                    perl -0777 -pe 's/^use ic_principal::Principal;$/use candid::Principal;/m;
                        s/^#\[cfg\(feature = "signature-map"\)\]\n(?=pub mod signature_map;$)//mg' \
                        "$candidate/$relative" > "$attempt/normalized.rs" ;;
                secp)
                    # Permit item-level scaffolding attributes only, never an
                    # attribute that changes statements inside a crypto body.
                    perl -0777 -pe '
                        my @lines = split /(?<=\n)/;
                        $_ = "";
                        for my $i (0 .. $#lines) {
                            if ($lines[$i] =~ /^[ \t]*#\[cfg\(feature = "full"\)\]$/) {
                                my $next = $lines[$i + 1] // "";
                                my $item = $next =~ /^(?:use (?:rand::|std::sync::LazyLock;|zeroize::ZeroizeOnDrop;)|static (?:ECDSA_OID|SECP256K1_OID):|const PEM_HEADER_|fn (?:der_|pem_encode|bip341_generate_tweak)|impl (?:PrivateKey|RecoveryId) |    (?:pub )?fn (?:deserialize_der|deserialize_pem|serialize_der|serialize_pem|verify_|try_recovery_from_digest|derive_bip341))/;
                                my $derived = $next =~ /^#\[derive\(/ &&
                                    ($lines[$i + 2] // "") =~ /^pub struct (?:PrivateKey|RecoveryId) /;
                                die "unexpected feature attribute location\n" unless $item || $derived;
                                next;
                            }
                            $_ .= $lines[$i];
                        }' \
                        "$candidate/$relative" > "$attempt/normalized.rs" ;;
            esac
            cmp "$source" "$attempt/normalized.rs"
        else
            cmp "$source" "$candidate/$relative"
        fi
        bash scripts/ci/verify-file-checksum.sh --print sha256 "$source" >> "$attempt/$directory.source-sha256"
    done < "$attempt/$directory.sources"
    # Refuse newly introduced source files, including a replacement implementation.
    rg --files "$candidate/src" | sed "s|^$candidate/||" | LC_ALL=C sort > "$attempt/candidate.files"
    sed "s|^$original/||" "$attempt/$directory.sources" | LC_ALL=C sort > "$attempt/original.files"
    cmp "$attempt/original.files" "$attempt/candidate.files"
    printf '%s: source preservation and license checks passed\n' "$package"
done
mkdir -p "$attempt/vectors"
cat > "$attempt/vectors/Cargo.toml" <<'MANIFEST'
[workspace]
members = ["original", "candidate"]
resolver = "3"
MANIFEST
for side in original candidate; do
    mkdir -p "$attempt/vectors/$side/src"
    cp scripts/fixtures/secp-scaffolding/main.rs "$attempt/vectors/$side/src/main.rs"
    dependency='version = "=0.3.0"'
    if [[ "$side" == candidate ]]; then
        dependency='version = "=0.3.0", path = "../../secp", default-features = false'
    fi
    cat > "$attempt/vectors/$side/Cargo.toml" <<MANIFEST
[package]
name = "vectors-$side"
version = "0.0.0"
edition = "2024"
[dependencies]
ic-secp256k1 = { $dependency }
MANIFEST
done
cp Cargo.lock "$attempt/vectors/Cargo.lock"
cargo metadata --manifest-path "$attempt/vectors/Cargo.toml" --format-version 1 \
    --filter-platform "$host" --offline > "$attempt/vectors/metadata.json"
printf 'Separate secp vector builds are prepared at %s/vectors\n' "$attempt"
printf 'Packaging proposals only; production dependency selections are unchanged.\n'
