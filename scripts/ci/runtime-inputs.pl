#!/usr/bin/env perl
# Bind runtime fixture copies to one captured input set, then detect source drift.
use strict;
use warnings;
use Digest::SHA qw(sha256_hex);
use File::Path qw(make_path);
use File::Basename qw(dirname);
my ($mode, $root, $fixture, $lock, $refresh) = @ARGV;
@ARGV >= 4 && @ARGV <= 5 && $mode =~ /\A(?:capture|verify)\z/
    && (!defined($refresh) || $refresh eq '--refresh-lock')
    or die "usage: runtime-inputs.pl capture|verify ROOT FIXTURE LOCK [--refresh-lock]\n";
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
    open my $rg, '-|', "$root/.tools/host/bin/rg", '--files', '-0', 'crates', 'scripts/fixtures/cdk-runtime' or die $!;
    local $/; my $raw = <$rg> // ''; close $rg or die "cannot inventory runtime inputs\n";
    my %names = map { $_ => 1 } grep { $_ ne 'scripts/fixtures/cdk-runtime/Cargo.lock' } split /\0/, $raw;
    $names{$_} = 1 for ('Cargo.toml', 'rust-toolchain.toml', '.cargo/config.toml',
        'scripts/ci/check-platform-runtime.sh', 'scripts/ci/runtime-inputs.pl',
        'scripts/ci/retain-build-artifact.sh', 'scripts/ci/render-runtime-fixture.pl',
        'scripts/ci/workspace-dependencies.sh', 'scripts/ci/check-platform-compiler.sh',
        'scripts/ci/read-platform-msrv.sh', $lock);
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
if (defined($refresh) && $mode eq 'verify') {
    # Only the selected maintained lock may change during explicit preparation.
    s/^[0-9a-f]{64}  \Q$lock\E\n//m for ($before, $after);
}
$before eq $after or die "Runtime qualification inputs changed during observation\n";
