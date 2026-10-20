#!/usr/bin/env bash
set -euo pipefail
# Compare the imported non-cryptographic call/cleanup machinery, not an executor fork.
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
if [[ $# == 0 ]]; then bash "$root/scripts/ci/check-signature-source.sh"; fi
source_root="${1:-$root/crates/ic-platform/src/cdk/upstream}"
[[ $# -le 1 ]] || exit 2
fixture="$(mktemp -d "${TMPDIR:-/tmp}/platform-runtime-source.XXXXXX")"
trap 'rm -rf "$fixture"' EXIT
for part in call spawn; do
    file="$source_root/call/mod.rs"
    [[ "$part" == call ]] || file="$source_root/futures/mod.rs"
    perl -0777 -e '
        use strict; use warnings;
        my ($part, $file) = @ARGV;
        open my $input, "<", $file or die $!; local $/; my $source = <$input>;
        my $body;
        if ($part eq "call") {
            ($body) = $source =~ m{(impl<.m, .a> IntoFuture for Call.*?)(?=// # Internal END)}s;
            $body =~ s/(?:crate|super)::api::/API::/g if defined $body;
        } else {
            ($body) = $source =~ m{(pub fn spawn<F:.*?)(?=/// Spawn a weak)}s;
        }
        die "missing qualified source region\n" unless defined $body;
        # Preserve string contents; ignore comments and formatting only.
        my @strings;
        $body =~ s{"(?:\\.|[^"\\])*"}{push(@strings, $&); "\x01" . $#strings . "\x02"}gse;
        $body =~ s{//[^\n]*}{}g;
        $body =~ s{/\*.*?\*/}{}gs;
        $body =~ s/\s+//g;
        $body =~ s/\x01(\d+)\x02/$strings[$1]/ge;
        print $body;
    ' "$part" "$file" > "$fixture/$part"
    expected="$("$root/.tools/host/bin/jq" -er --arg part "$part" '.qualified_regions[$part]' "$root/crates/ic-platform/src/cdk/upstream/provenance.json")"
    bash "$root/scripts/ci/verify-file-checksum.sh" sha256 "$expected" "$fixture/$part"
done
expected="$("$root/.tools/host/bin/jq" -er '.license_sha256' "$root/crates/ic-platform/src/cdk/upstream/provenance.json")"
bash "$root/scripts/ci/verify-file-checksum.sh" sha256 "$expected" "$source_root/LICENSE-APACHE"
printf 'Imported call submission, callback/trap cleanup and protected-spawn source regions match CDK 0.20.3.\n'
