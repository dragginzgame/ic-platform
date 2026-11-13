#!/usr/bin/env bash
# Read the root catalog once; preserve complete dependency tables in fixtures.
platform_dependencies_read() {
    # Archived catalogs contain source, not an installed tool bundle. Use the
    # invoking helper's admitted tools while reading the selected source bytes.
    platform_dependency_tools="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)/.tools/host/bin"
    platform_dependencies="$("$platform_dependency_tools/yq" -p toml -o json '.workspace.dependencies' "$1/Cargo.toml")"
}
platform_dependency() {
    # shellcheck disable=SC2016 # $name is a jq binding.
    "$platform_dependency_tools/jq" -er --arg name "$1" '
        .[$name] | if type == "string" then {version: .}
        elif type == "object" then . else error("missing dependency: " + $name) end |
        to_entries | map("\(.key) = \(.value | tojson)") | join(", ")' <<< "$platform_dependencies"
}
platform_dependency_version() {
    # shellcheck disable=SC2016 # $name is a jq binding.
    "$platform_dependency_tools/jq" -er --arg name "$1" '
        .[$name] | if type == "object" then .version else . end |
        if type == "string" and length > 0 then . else error("missing dependency version: " + $name) end' <<< "$platform_dependencies"
}

# Generated consumers use this repository's compiler/lint policy, not their
# dependency's table (Cargo deliberately does not inherit dependency lints).
platform_fixture_policy() {
    local owner="$1" directory="$2" tools configuration compiler manifest member data
    [[ "$directory" != "$owner" ]] || { echo 'Fixture policy cannot rewrite its owning workspace' >&2; return 1; }
    tools="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)/.tools/host/bin"
    configuration="$("$tools/yq" -p toml -o json '.workspace.lints' "$owner/Cargo.toml")"
    "$tools/jq" -e '(.rust|type=="object") and (.rustdoc|type=="object") and (.clippy|type=="object")' <<< "$configuration" > /dev/null || return
    compiler="$(bash "$(dirname "${BASH_SOURCE[0]}")/read-platform-msrv.sh" "$owner/Cargo.toml")"
    data="$("$tools/yq" -p toml -o json '.' "$directory/Cargo.toml")"
    if [[ "$("$tools/jq" -r '.workspace.lints.rust.warnings // ""' <<< "$data")" == deny ]]; then
        configuration="$("$tools/jq" '.rust.warnings="deny"' <<< "$configuration")"
    fi
    # These owned templates declare literal members. Refuse to silently miss a
    # member if a future template introduces Cargo glob/exclusion semantics.
    "$tools/jq" -e '(.workspace.exclude // [] | length)==0 and all(.workspace.members // [] | .[]; test("^[A-Za-z0-9_./-]+$") and (contains("..")|not))' <<< "$data" > /dev/null || return
    local manifests=()
    if "$tools/jq" -e 'has("package")' <<< "$data" > /dev/null; then manifests+=("$directory/Cargo.toml"); fi
    while IFS= read -r member; do
        [[ -z "$member" ]] || manifests+=("$directory/$member/Cargo.toml")
    done < <("$tools/jq" -r '.workspace.members // [] | .[]' <<< "$data")
    for manifest in ${manifests[@]+"${manifests[@]}"}; do
        [[ -f "$manifest" && ! -L "$manifest" ]] || return 1
        # Preserve the previous strict Rust-warning contract where selected.
        if [[ "$("$tools/yq" -p toml -o json '.lints.rust.warnings' "$manifest")" == '"deny"' ]]; then
            configuration="$("$tools/jq" '.rust.warnings="deny"' <<< "$configuration")"
        fi
    done
    PLATFORM_FIXTURE_LINTS="$configuration" perl - "$directory/Cargo.toml" "$compiler" ${manifests[@]+"${manifests[@]}"} <<'POLICY'
use strict; use warnings; use JSON::PP;
my ($root, $compiler, @members)=@ARGV;
my %members=map {$_=>1} @members;
my $lints=decode_json($ENV{PLATFORM_FIXTURE_LINTS});
my $json=JSON::PP->new->canonical;
sub scalar_toml {
    my ($value)=@_;
    return $json->encode($value) unless ref $value;
    ref($value) eq 'HASH' or die "unsupported lint value\n";
    return '{'.join(', ',map {$json->encode($_).' = '.scalar_toml($value->{$_})} sort keys %$value).'}';
}
for my $file (sort keys %{ { %members, $root=>1 } }) {
    -f $file && !-l $file or die "missing fixture manifest $file\n";
    open my $in,'<',$file or die $!;
    my ($table,$skip,$floor)=('',0,0); my $text='';
    while (my $line=<$in>) {
        if ($line =~ /^\[([^\]]+)\]\s*$/) {
            $table=$1; $skip=$table =~ /\A(?:workspace\.)?lints(?:\.|\z)/;
            next if $skip;
            $text.=$line;
            $text.="rust-version.workspace = true\n" if $table eq 'package';
            if ($file eq $root && $table eq 'workspace.package') {
                $text.="rust-version = \"$compiler\"\n"; $floor=1;
            }
            next;
        }
        next if $skip || ($table =~ /\A(?:workspace\.)?package\z/ && $line =~ /^rust-version(?:\.workspace)?\s*=/);
        $text.=$line;
    }
    close $in or die $!;
    $text =~ s/\s+\z/\n/;
    $text.="\n[workspace.package]\nrust-version = \"$compiler\"\n" if $file eq $root && !$floor;
    $text.="\n[lints]\nworkspace = true\n" if $members{$file};
    if ($file eq $root) {
        for my $group (sort keys %$lints) {
            $text.="\n[workspace.lints.$group]\n";
            $text.=$_.' = '.scalar_toml($lints->{$group}{$_})."\n" for sort keys %{$lints->{$group}};
        }
    }
    open my $out,'>',$file or die $!; print {$out} $text; close $out or die $!;
}
POLICY
}
