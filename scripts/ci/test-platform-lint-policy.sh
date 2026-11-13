#!/usr/bin/env bash
# Actual inherited policy, not a substitute compiler or a metadata-only proof.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
bash scripts/ci/check-platform-compiler.sh
# shellcheck source=scripts/ci/workspace-dependencies.sh
source "$root/scripts/ci/workspace-dependencies.sh"
mkdir -p target
fixture="$(mktemp -d "$root/target/lint-policy.XXXXXX")"
trap 'printf "Compiler/lint policy evidence: %s\n" "$fixture"' EXIT
for workspace in "$root" "$root/qualification"; do
    cargo metadata --manifest-path "$workspace/Cargo.toml" --no-deps --locked --offline --format-version 1 > "$fixture/members.json"
    while IFS= read -r manifest; do
        [[ "$("$root/.tools/host/bin/yq" -p toml -o json '.lints.workspace' "$manifest")" == true ]]
    done < <(jq -r '.packages[].manifest_path' "$fixture/members.json")
done
mkdir "$fixture/src"
printf '[workspace]\n[package]\nname="platform-lint-contract"\nversion="0.0.0"\nedition="2024"\n' > "$fixture/Cargo.toml"
platform_fixture_policy "$root" "$fixture"
cp "$fixture/Cargo.toml" "$fixture/first.toml"
platform_fixture_policy "$root" "$fixture"
cmp "$fixture/first.toml" "$fixture/Cargo.toml"
printf '//! Positive compiler-policy consumer.\n/// Return the observed value.\n#[must_use]\npub fn value()->u8{1}\n' > "$fixture/src/lib.rs"
for target in "$(rustc -vV | sed -n 's/^host: //p')" wasm32-unknown-unknown; do
    cargo check --manifest-path "$fixture/Cargo.toml" --offline --target "$target" > "$fixture/$target.log" 2>&1
done
cargo clippy --manifest-path "$fixture/Cargo.toml" --locked --offline -- -D warnings > "$fixture/clippy.log" 2>&1
cargo doc --manifest-path "$fixture/Cargo.toml" --locked --offline --no-deps > "$fixture/rustdoc.log" 2>&1
for case in must-use unsafe unsafe-op correctness; do
    case "$case" in
        must-use) code='pub fn exercise(){Result::<(),()>::Ok(());}' ; diagnostic=unused_must_use ;;
        unsafe) code='pub fn exercise(){unsafe{}}' ; diagnostic=unsafe_code ;;
        unsafe-op) code='#[expect(unsafe_code, reason="Separate unsafe-op contract fixture")] pub unsafe fn exercise(p:*const u8)->u8{*p}' ; diagnostic=E0133 ;;
        correctness) code='pub fn exercise(x:u8)->bool{x==x}' ; diagnostic=clippy::eq_op ;;
    esac
    printf '//! Deliberately rejected policy fixture.\n/// Exercise the selected refusal.\n%s\n' "$code" > "$fixture/src/lib.rs"
    command=(cargo check)
    if [[ "$case" == correctness ]]; then command=(cargo clippy); fi
    if "${command[@]}" --manifest-path "$fixture/Cargo.toml" --locked --offline --message-format=json > "$fixture/$case.jsonl" 2> "$fixture/$case.stderr"; then
        echo "Compiler admitted forbidden $case fixture" >&2; exit 1
    fi
    jq -se --arg diagnostic "$diagnostic" 'any(.[]; .reason=="compiler-message" and .message.level=="error" and .message.code.code==$diagnostic)' "$fixture/$case.jsonl" > /dev/null
done
printf '//! Broken documentation refusal.\n/// Return [not_an_item].\n#[must_use]\npub fn value()->u8{1}\n' > "$fixture/src/lib.rs"
if cargo doc --manifest-path "$fixture/Cargo.toml" --locked --offline --no-deps > "$fixture/broken-doc.log" 2>&1; then
    echo 'Rustdoc admitted a broken intra-doc link' >&2; exit 1
fi
grep -Eq 'broken_intra_doc_links|unresolved link' "$fixture/broken-doc.log"
printf 'Inherited compiler/lint policy: native/Wasm positives and must-use, unsafe, unsafe-op, correctness and Rustdoc refusals passed.\n'
