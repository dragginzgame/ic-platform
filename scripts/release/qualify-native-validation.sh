#!/usr/bin/env bash
# Broad acceptance only: a fresh disposable source and synthetic history exercise
# real full/selected mappings. Nothing is pushed or admitted as a release baseline.
set -euo pipefail
[[ $# == 0 ]] || exit 2
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
[[ -z "$(git status --porcelain --untracked-files=all)" ]] || {
    echo 'Native mapping qualification requires clean committed input.' >&2; exit 1;
}
[[ -n "${POCKET_IC_BIN:-}" || -n "${IC_TESTKIT_POCKET_IC_URL:-}" ]] || exit 1
mkdir -p target
attempt="$(mktemp -d "$root/target/release-validation-native.XXXXXX")"
mkdir "$attempt/evidence"
retain() {
    status=$?
    if [[ -d "$attempt/repo/target/release-validation" ]]; then
        cp -R "$attempt/repo/target/release-validation" "$attempt/evidence/validation"
    fi
    for directory in "$attempt/repo"/target/cdk-runtime.*; do
        [[ -d "$directory" ]] || continue
        destination="$attempt/evidence/$(basename "$directory")"
        mkdir -p "$destination"
        for name in qualification operation status source-revision source.sha256 compiler.txt artifacts.sha256 profiles.log; do
            if [[ -f "$directory/$name" ]]; then cp "$directory/$name" "$destination/"; fi
        done
    done
    printf 'Native mapping evidence: %s\n' "$attempt/evidence"
    exit "$status"
}
trap retain EXIT
git rev-parse HEAD > "$attempt/evidence/input-source"
git clone --quiet --shared --no-hardlinks "$root" "$attempt/repo"
git -C "$attempt/repo" remote remove origin
for path in .tools packages/browser/node_modules scripts/fixtures/browser-compiler-floor/node_modules; do
    [[ -d "$root/$path" && ! -L "$root/$path" ]] || exit 1
    cp -R "$root/$path" "$attempt/repo/$path"
done
cd "$attempt/repo"
git config user.name 'Platform native qualification'
git config user.email qualification@example.invalid
for group in platform runtime types; do git tag "$group-v99.0.0"; done
printf '\nIsolated native full/selected mapping qualification.\n' >> docs/status/current.md
git add docs/status/current.md
git commit -qm 'Controlled same-product-source validation baseline'
git rev-parse HEAD > "$attempt/evidence/qualified-source"
# Preserve the synthetic commit itself, with the recorded real input as its
# prerequisite, so uploaded receipts can be reconstructed after runner cleanup.
git bundle create "$attempt/evidence/source.bundle" HEAD "^$(cat "$attempt/evidence/input-source")"
perl scripts/release/validation-context.pl > "$attempt/evidence/execution.json"
bash scripts/release/qualify-validation-plan.sh platform platform-v99.0.0 workspace > "$attempt/evidence/full.log" 2>&1
for group in runtime types; do
    bash scripts/release/qualify-validation-plan.sh "$group" "$group-v99.0.0" auto > "$attempt/evidence/$group.log" 2>&1
done
perl scripts/release/validation-plan.pl platform "$(git rev-parse HEAD)" platform-v99.0.0 target/release-validation/facade-auto.json auto
jq -e '.scope=="workspace" and ([.stages[].required] | all)' target/release-validation/facade-auto.json
perl -MJSON::PP -e '
    my %groups;
    my %expected=(platform=>20,runtime=>18,types=>19);
    my ($source,$host,$baseline);
    for my $file (@ARGV) {
        open my $fh,"<",$file or die $!; local $/; my $r=decode_json(<$fh>);
        die "duplicate group" if $groups{$r->{group}}++;
        $source //= $r->{source}; $host //= $r->{host_key}; $baseline //= $r->{baseline};
        die "different source/host/baseline" unless $r->{source} eq $source && $r->{host_key} eq $host && $r->{baseline} eq $baseline;
        die "wrong scope" unless $r->{scope} eq ($r->{group} eq "platform" ? "workspace" : "group");
        my $seconds=0; my $stages=0;
        for (@{$r->{validation_evidence}{stages}}) { next if $_->{state} eq "not-applicable"; die "not passed" unless $_->{state} eq "PASS"; $seconds+=$_->{seconds}; $stages++; }
        die "unexpected stage count" unless exists($expected{$r->{group}}) && $stages==$expected{$r->{group}};
        printf "%s\t%s\t%s\t%d\t%d\n",$r->{source},$r->{host_key},$r->{group},$stages,$seconds;
    }
    die "missing group" unless keys(%groups)==3;
' target/release-validation/qualification.*/receipt.json > "$attempt/evidence/results.tsv"
cat "$attempt/evidence/results.tsv"
