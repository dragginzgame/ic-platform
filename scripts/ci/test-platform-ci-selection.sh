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
    my $native_steps=$jobs->{"native-provider-source"}{steps};
    my ($tools)=grep { ($native_steps->[$_]{run}//"") eq "make install-host-tools" } 0..$#$native_steps;
    my ($prepare)=grep { ($native_steps->[$_]{run}//"") eq "bash scripts/ci/check-native-profiler-http.sh --prepare" } 0..$#$native_steps;
    defined($tools) && defined($prepare) && $tools < $prepare or die "native preparation lacks required manifest tools";
    print $classify->{run};
' "$fixture/workflow.json" > "$fixture/classify.sh"
mkdir -p "$fixture/repo/ci" "$fixture/repo/scripts/ci" "$fixture/repo/docs/status" "$fixture/repo/crates/example"
cp "$root/ci/docs-only-paths.txt" "$fixture/repo/ci/"
bash "$root/scripts/ci/prepare-script-fixture.sh" "$root" "$fixture/repo" scripts/ci/classify-ci-change.pl
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

# The repository owns one asynchronous scan and restores/saves authenticated
# downloads around each actual host/CI/IC tool setup.
workflows=()
for workflow in "$root"/.github/workflows/*.yml; do
    rendered="$fixture/$(basename "$workflow").json"
    "$yq" -o json . "$workflow" > "$rendered"
    workflows+=("$rendered")
done
perl -MJSON::PP -e '
    my ($scans,$installs)=(0,0);
    for my $file (@ARGV) {
        open my $input,"<",$file or die "$file: $!";
        local $/; my $workflow=decode_json(<$input>);
        for my $name (keys %{$workflow->{jobs}}) {
            my $job=$workflow->{jobs}{$name}; my $steps=$job->{steps}//[];
            my (@restore,@save,@setup,@scan);
            for my $i (0..$#$steps) {
                my $step=$steps->[$i]; my $run=$step->{run}//"";
                if (($step->{uses}//"") eq "./.github/actions/cache-tool-downloads") {
                    push @{(($step->{with}{mode}//"restore") eq "save") ? \@save : \@restore}, $i;
                }
                if ($run =~ /make install-(?:host-tools|tools)\b|install-ci-tool\.sh gitleaks/) { push @setup,$i; }
                $installs++ if $run =~ /install-ci-tool\.sh gitleaks/;
                push @scan,$i if $run =~ /GITLEAKS_BIN.* git /;
            }
            if (@setup) {
                @restore==1 && @save==1 && $restore[0]<$setup[0] && $save[0]>$setup[-1]
                    or die "$file / $name: missing ordered download cache setup";
            }
            for my $i (@scan) {
                $scans++;
                !exists($job->{strategy}) && !exists($job->{needs}) && !exists($job->{if})
                    or die "secret scan must be independent of selection/matrices";
                my ($checkout)=grep { ($_->{uses}//"") =~ m{^actions/checkout\@} } @$steps;
                ($checkout->{with}{"fetch-depth"}//-1)==0 or die "secret scan needs full history";
                ($steps->[$i]{run}//"") eq q{"$GITLEAKS_BIN" git --redact=100 --no-banner --no-color --log-level error}
                    or die "secret scan command drift";
                !exists($steps->[$i]{if}) && !exists($steps->[$i]{"continue-on-error"})
                    or die "secret scan must run/fail for documentation changes too";
            }
        }
    }
    $scans==1 && $installs==1 or die "expected exactly one scan/installer across workflows";
    print "Repository scan ownership and hosted download-cache ordering passed\n";
' "${workflows[@]}"
