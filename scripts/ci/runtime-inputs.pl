#!/usr/bin/env perl
# Bind runtime fixture copies to one captured input set, then detect source drift.
use strict;
use warnings;
use Digest::SHA qw(sha256_hex);
use File::Path qw(make_path);
use File::Basename qw(dirname);
my ($mode, $root, $fixture, $lock, $option) = @ARGV;
@ARGV >= 4 && @ARGV <= 5 && $mode =~ /\A(?:capture|verify)\z/
    && (!defined($option) || $option =~ /\A--(?:refresh-lock|raw-rand)\z/)
    or die "usage: runtime-inputs.pl capture|verify ROOT FIXTURE LOCK [--refresh-lock|--raw-rand]\n";
my $raw_rand = ($option // '') eq '--raw-rand';
!$raw_rand || $lock eq 'Cargo.lock' or die "raw_rand uses the workspace lock\n";
chdir $root or die "$root: $!\n";
sub read_file {
    my ($base, $path) = @_;
    $path !~ m{\A/|(?:\A|/)\.{1,2}(?:/|\z)|[\x00-\x1f]}
        or die "unsafe runtime input path\n";
    my $route = $base;
    for (split m{/}, $path) { $route .= "/$_"; !-l $route or die "redirected runtime input $path\n"; }
    open my $in, '<:raw', "$base/$path" or die "$path: $!\n";
    local $/; my $bytes = <$in> // ''; close $in or die $!;
    return $bytes;
}
sub write_file {
    my ($path, $bytes) = @_;
    make_path(dirname($path));
    open my $out, '>:raw', $path or die "$path: $!\n";
    print {$out} $bytes; close $out or die $!;
}
sub names {
    my @roots = $raw_rand ? qw(crates scripts/fixtures/raw-rand scripts/fixtures/cdk-runtime/pocketic_support)
        : qw(crates scripts/fixtures/cdk-runtime);
    open my $rg, '-|', "$root/.tools/host/bin/rg", '--files', '-0', @roots or die $!;
    local $/; my $raw = <$rg> // ''; close $rg or die "cannot inventory runtime inputs\n";
    my %names = map { $_ => 1 } grep { $_ ne 'scripts/fixtures/cdk-runtime/Cargo.lock' } split /\0/, $raw;
    $names{$_} = 1 for ('Cargo.toml', 'rust-toolchain.toml', '.cargo/config.toml',
        'scripts/ci/check-platform-runtime.sh', 'scripts/ci/runtime-inputs.pl',
        'scripts/ci/runtime-selection.sh',
        'scripts/ci/retain-build-artifact.sh', 'scripts/ci/render-runtime-fixture.pl',
        'scripts/ci/workspace-dependencies.sh', 'scripts/ci/check-platform-compiler.sh',
        'scripts/ci/read-platform-msrv.sh', $lock);
    if ($raw_rand) {
        $names{$_} = 1 for qw(scripts/ci/check-platform-raw-rand.sh scripts/ci/check-platform-runtime-graph.sh
            scripts/ci/platform-graph-policy.awk scripts/benchmarks/wasm-sections.pl
            scripts/fixtures/cdk-runtime/raw_rand_driver.rs);
    }
    return sort keys %names;
}
sub hashes {
    my ($base, @names) = @_;
    return join '', map { sha256_hex(read_file($base, $_)) . "  $_\n" } @names;
}
if ($mode eq 'capture') {
    !-e "$fixture/inputs" && !-l "$fixture/inputs" or die "occupied runtime input snapshot\n";
    my @names = names();
    my $hashes = '';
    for my $path (@names) {
        my $bytes = read_file($root, $path);
        write_file("$fixture/inputs/$path", $bytes);
        $hashes .= sha256_hex($bytes) . "  $path\n";
    }
    write_file("$fixture/input-files", join('', map { "$_\n" } @names));
    write_file("$fixture/source.sha256", $hashes);
}
my $before = read_file($fixture, 'source.sha256');
my @names = names();
my $roster = join '', map { "$_\n" } @names;
$roster eq read_file($fixture, 'input-files') or die "Runtime qualification input roster changed\n";
hashes("$fixture/inputs", @names) eq $before or die "Runtime input snapshot changed\n";
my $after = hashes($root, @names);
write_file("$fixture/source-after.sha256", $after);
if (($option // '') eq '--refresh-lock' && $mode eq 'verify') {
    # Only the selected maintained lock may change during explicit preparation.
    s/^[0-9a-f]{64}  \Q$lock\E\n//m for ($before, $after);
}
$before eq $after or die "Runtime qualification inputs changed during observation\n";
