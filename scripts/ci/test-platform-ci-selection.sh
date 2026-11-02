#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
mkdir -p "$root/target"
fixture="$(mktemp -d "$root/target/ci-selection.XXXXXX")"
trap 'printf "CI selection evidence: %s\n" "$fixture"' EXIT
yq="$root/.tools/host/bin/yq"
"$yq" -o json . "$root/.github/workflows/ci.yml" > "$fixture/workflow.json"
# Inspect the actual consumer wiring, including fallback and native hosts.
perl -MJSON::PP -0777 -e '
    my $w=decode_json(<>); my $jobs=$w->{jobs};
    die "workflow path filter" if exists $w->{on}{push}{paths} || exists $w->{on}{pull_request}{paths};
    my $selection=$jobs->{selection};
    $selection->{steps}[0]{with}{"fetch-depth"} == 0 or die "incomplete history";
    my ($classify)=grep { ($_->{id}//"") eq "classify" } @{$selection->{steps}};
    $classify->{env}{CHANGE_HEAD} eq q{${{ github.sha }}} or die "untested head";
    $classify->{env}{CHANGE_BASE} eq q{${{ github.event.pull_request.base.sha || github.event.before }}} or die "wrong base";
    $classify->{env}{CHANGE_REF} eq q{${{ github.event_name == '\''pull_request'\'' && format('\''refs/heads/{0}'\'', github.base_ref) || github.ref }}} or die "wrong ref";
    for my $name (qw(platform native-provider-source)) {
        my $j=$jobs->{$name};
        $j->{needs} eq "selection" && $j->{if} eq q{${{ !cancelled() }}} or die "missing failure fallback/cancellation";
        my ($docs,@full);
        for my $s (@{$j->{steps}}) {
            next if ($s->{uses}//"") =~ m{^actions/checkout@} || ($s->{name}//"") eq "Check native identity";
            if (($s->{if}//"") eq q{needs.selection.outputs.scope == '\''docs'\''}) {
                $s->{run} eq "make shared-tooling-check check-doc-links" or die "missing real docs/snapshot checks";
                $docs++;
            } else {
                ($s->{if}//"") eq q{needs.selection.outputs.scope != '\''docs'\''} or die "unsafe full selection";
                push @full,$s;
            }
        }
        $docs == 1 && @full or die "missing qualification";
    }
    my @hosts=sort map { $_->{runner} } @{$jobs->{platform}{strategy}{matrix}{include}};
    join(",",@hosts) eq "macos-15,macos-15-intel,ubuntu-24.04" or die "lost native host";
    print $classify->{run};
' "$fixture/workflow.json" > "$fixture/classify.sh"
mkdir -p "$fixture/repo/ci" "$fixture/repo/scripts/ci" "$fixture/repo/docs/status" "$fixture/repo/crates/example"
cp "$root/ci/docs-only-paths.txt" "$fixture/repo/ci/"
cp "$root/scripts/ci/classify-ci-change.pl" "$fixture/repo/scripts/ci/"
cd "$fixture/repo"
git init -q
git config user.name 'CI selection fixture'
git config user.email 'fixture@example.invalid'
printf 'Initial prose\n' > docs/status/current.md
printf 'fn original() {}\n' > crates/example/lib.rs
git add ci scripts docs crates
git commit -qm baseline
base="$(git rev-parse HEAD)"
check() {
    local expected="$1" event="$2" start="$3" head="$4" ref="$5"
    : > "$fixture/output"
    CHANGE_EVENT="$event" CHANGE_REF="$ref" CHANGE_BASE="$start" CHANGE_HEAD="$head" GITHUB_OUTPUT="$fixture/output" \
        bash "$fixture/classify.sh" > "$fixture/last.log" 2>&1
    [[ "$(cat "$fixture/output")" == "scope=$expected" ]] || { cat "$fixture/last.log" >&2; exit 1; }
}
printf 'More prose\n' >> docs/status/current.md
git commit -qam docs
docs="$(git rev-parse HEAD)"
check docs push "$base" "$docs" refs/heads/main
check docs pull_request "$base" "$docs" refs/heads/main
check full push "$base" "$docs" refs/tags/v1
check full push 0000000000000000000000000000000000000000 "$docs" refs/heads/main
check full push "$docs" "$docs" refs/heads/main
printf 'fn changed() {}\n' >> crates/example/lib.rs
git commit -qam code
code="$(git rev-parse HEAD)"
check full push "$docs" "$code" refs/heads/main
check full pull_request "$base" "$code" refs/heads/main
printf '# changed policy\n' >> ci/docs-only-paths.txt
git commit -qam policy
check full push "$code" "$(git rev-parse HEAD)" refs/heads/main
# A classifier process failure must not produce a docs output. The workflow
# assertions above prove both required jobs retain full qualification on failure.
mv scripts/ci/classify-ci-change.pl scripts/ci/classifier-unavailable
: > "$fixture/output"
if CHANGE_EVENT=push CHANGE_REF=refs/heads/main CHANGE_BASE="$base" CHANGE_HEAD="$docs" GITHUB_OUTPUT="$fixture/output" \
    bash "$fixture/classify.sh" > "$fixture/failed-observation.log" 2>&1; then
    echo 'Classifier failure was swallowed' >&2; exit 1
fi
[[ ! -s "$fixture/output" ]]
echo 'Actual CI wiring preserves native hosts and fails closed for code, mixed, policy, missing-history and failed observations.'
