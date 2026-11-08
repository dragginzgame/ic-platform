#!/usr/bin/env perl
# Render the three explicit qualification manifests once, without post-render edits.
use strict;
use warnings;
my ($source, $fixture, $qualification, $selection, $adapter) = @ARGV;
@ARGV == 5 && $qualification =~ /\A(?:owner|timers)\z/
    && $selection =~ /\A(?:source|published)\z/ && $adapter =~ /\A(?:raw|external)\z/
    or die "usage: render-runtime-fixture.pl TEMPLATE-DIR FIXTURE owner|timers source|published raw|external\n";
my %values;
for my $name (qw(RUNTIME PLATFORM TESTKIT WIRE SERDE TOKIO MSRV)) {
    $values{$name} = $ENV{"PLATFORM_FIXTURE_$name"} // die "missing $name\n";
}
$values{TIMERS_ROOT} = '';
$values{TIMERS_CANISTER} = '';
if ($qualification eq 'timers') {
    my $timers = $ENV{PLATFORM_FIXTURE_TIMERS} // die "missing TIMERS\n";
    $values{TIMERS_ROOT} = "ic-timers = { $timers }";
    $values{TIMERS_CANISTER} = 'ic-timers.workspace = true';
}
$values{EXTERNAL_BRIDGE} = $adapter eq 'external' ? ', "ic-platform/candid-endpoint"' : '';
# Only qualification-local Runtime owners are patched; the facade stays independent.
$values{RUNTIME_PATCH} = $selection eq 'published' ? '' : <<'PATCH';
[patch.crates-io]
ic-platform-runtime = { path = "../../crates/ic-platform-runtime" }
ic-platform-macros = { path = "../../crates/ic-platform-macros" }
ic-platform-runtime-guard = { path = "../../crates/ic-platform-runtime-guard" }
PATCH
for my $entry (
    ['Cargo.toml.in', 'Cargo.toml'],
    ['canister.Cargo.toml.in', 'crates/canister/Cargo.toml'],
    ['driver.Cargo.toml', 'apps/driver/Cargo.toml'],
) {
    open my $input, '<', "$source/$entry->[0]" or die "$source/$entry->[0]: $!\n";
    local $/; my $text = <$input>;
    $text =~ s/\@([A-Z_]+)\@/exists($values{$1}) ? $values{$1} : die("unknown placeholder $1\n")/ge;
    open my $output, '>', "$fixture/$entry->[1]" or die "$fixture/$entry->[1]: $!\n";
    print {$output} $text;
    close $output or die "close $fixture/$entry->[1]: $!\n";
}
