#!/usr/bin/env perl
# Consumer-owned selection only; execution/checkpointing belongs to Shared.
use strict;
use warnings;
use JSON::PP;
my $json = JSON::PP->new->canonical->utf8;
my ($group, $source, $baseline, $output, $scope) = @ARGV;
@ARGV == 5 && $scope =~ /\A(?:auto|workspace|release)\z/
    or die "usage: validation-plan.pl GROUP SOURCE GROUP-TAG|none OUTPUT auto|workspace|release\n";
sub capture {
    open my $fh, '-|', @_ or die "start @_: $!\n";
    local $/; my $value = <$fh> // '';
    close $fh or die "command failed: @_\n";
    return $value;
}
sub read_file {
    my ($file) = @_;
    $file !~ m{^/|(?:^|/)\.{1,2}(?:/|$)|[\x00-\x1f]|//} or die "unsafe input path\n";
    my $route = '.';
    for (split m{/}, $file) { $route .= "/$_"; !-l $route or die "redirected input $file\n"; }
    -f $file or die "non-file input $file\n";
    open my $fh, '<:raw', $file or die "read $file: $!\n";
    local $/; return <$fh>;
}
sub same_names {
    my ($a,$b) = @_;
    join("\0",@$a) eq join("\0",@$b) or die "stage/group roster drift\n";
}
my $policy_path = 'ci/release-validation.json';
my $policy_raw = read_file($policy_path);
my $policy = $json->decode($policy_raw);
$policy_raw eq $json->encode($policy)."\n" or die "noncanonical policy\n";
my $groups = $json->decode(read_file('ci/cargo-release-groups.json'));
my @groups = sort map { $_->{name} } @{$groups->{groups}};
same_names($policy->{groups}, \@groups);
grep($_ eq $group, @groups) or die "unknown group\n";
my $make = read_file('Makefile');
my ($ci) = $make =~ /^ci:\n\t\+bash scripts\/ci\/run-validation-targets\.sh --fail-fast ([^\n]+)\n/m;
my ($runtime) = $make =~ /^check-runtime-all:\n\t\+bash scripts\/ci\/run-validation-targets\.sh ([^\n]+)\n/m;
defined($ci) && defined($runtime) or die "full gate recipe changed; requalify roster\n";
$make =~ /^release-verify:\n\t\+bash scripts\/release\/validation-gate\.sh verify\n/m
    or die "release gate changed; requalify roster\n";
my @full = (split(/ /,$ci), split(/ /,$runtime));
same_names($policy->{stages}, \@full);
$source =~ /\A[0-9a-f]{40,64}\z/ && capture('git','rev-parse','HEAD') eq "$source\n"
    or die "stale release source\n";
capture('git','status','--porcelain','--untracked-files=all') eq ''
    or die "release plan requires clean committed source\n";
$baseline eq 'none' || $baseline =~ /\A\Q$group\E-v\d+\.\d+\.\d+\z/ or die "wrong baseline group/tag\n";
my $base = $baseline eq 'none' ? $source : capture('git','rev-parse','--verify',"refs/tags/$baseline^{commit}"); chomp $base;
system('git','merge-base','--is-ancestor',$base,$source) == 0 or die "baseline is not ancestor\n";
my @changed = grep { length } split /\0/, capture('git','diff','--name-only','--no-renames','-z',$base,$source,'--');
# Narrow only source/test changes within one existing owner. Manifest, lock,
# generator, imported-source/provenance, cross-group and unknown paths expand.
my %owners = (platform=>['ic-platform'], 'pocket-ic'=>['ic-platform-pocket-ic'],
    runtime=>['ic-platform-runtime','ic-platform-macros','ic-platform-runtime-guard'], types=>['ic-platform-types']);
my $owned = join '|', map { quotemeta } @{$owners{$group}};
my $narrow = @changed && $scope ne 'workspace' && $baseline ne 'none';
my $execution = $json->decode(capture('perl','scripts/release/validation-context.pl'));
my $hosts = $json->decode(read_file('ci/release-validation-hosts.json'));
$hosts->{schema} == 1 && ref($hosts->{qualified}) eq 'ARRAY' or die "invalid native qualification policy\n";
my %seen_hosts;
for (@{$hosts->{qualified}}) {
    /\A(?:Linux x86_64|Darwin (?:x86_64|arm64) 15)\z/ && !$seen_hosts{$_}++
        or die "unknown or duplicate qualified native host\n";
}
my $release;
if ($scope eq 'release') {
    $release = {map {$_ => $ENV{"RELEASE_".uc($_)}} qw(previous version remote branch)};
    for (qw(previous version)) {
        defined($release->{$_}) && $release->{$_} =~ /\A\d+\.\d+\.\d+\z/ or die "missing release version identity\n";
    }
    for (qw(remote branch)) { defined($release->{$_}) && length($release->{$_}) or die "missing release destination\n"; }
    $baseline eq 'none' || $baseline eq "$group-v$release->{previous}" or die "wrong release baseline\n";
    $narrow = 0 unless grep { $_ eq $execution->{host_key} } @{$hosts->{qualified}};
}
for my $path (@changed) {
    if ($path =~ m{\Acrates/ic-platform-(?:macros|runtime-guard)/}) { $narrow = 0; next; }
    next if $path =~ m{\Acrates/(?:$owned)/(?:src|tests)/[^\x00-\x1f]+\.rs\z};
    next if $path eq "releases/$group/CHANGELOG.md" || $path eq 'docs/status/current.md';
    $narrow = 0;
}
# Only the independently authored client has no Rust-to-browser schema surface.
# Full compile/lint/test/consumer/feature checks deliberately stay applicable.
my %excluded = (
    'check-release-commands' => 'Only owner Rust sources/tests and its notes changed; all release scripts, recipes, group metadata and qualification inputs are byte-identical to the baseline.',
);
if ($group eq 'pocket-ic' || $group eq 'runtime') {
    $excluded{'check-browser'} = 'Browser package, JS dependencies, browser fixtures and Types/schema inputs are unchanged; native client/runtime implementation has no browser-package code path.';
}
# Keep both installed suites even for client-only changes: their drivers use the
# client. Types/Runtime public identity and singleton boundaries also require them.
# Facade changes remain full until crypto/external-adapter scopes are qualified.
$narrow = 0 if $group eq 'platform';
my $index_inventory = capture('git','ls-files','--stage','-z');
my (@inputs, @gitlinks);
for my $entry (grep { length } split /\0/, $index_inventory) {
    my ($mode,$object,$stage,$path) = $entry =~ /\A(\d+) ([0-9a-f]+) (\d+)\t(.+)\z/s;
    defined($path) && $stage == 0 or die "unmerged/invalid source inventory\n";
    if ($mode eq '160000') {
        # Native IC source is a pinned external input, not read by these stages.
        # Bind its gitlink; do not walk a sibling/submodule working checkout.
        push @gitlinks, {path=>$path,object=>$object};
    } else {
        $mode eq '100644' || $mode eq '100755' or die "redirected tracked input\n";
        read_file($path); push @inputs,$path;
    }
}
my $context = {
    schema=>1, source=>$source, group=>$group, baseline=>$base, baseline_tag=>$baseline,
    changed=>\@changed, input_inventory=>\@inputs, gitlinks=>\@gitlinks, index_inventory=>$index_inventory,
    qualification=>defined($release)?'release':'qualification only', release=>$release,
    %$execution,
};
my $context_path = "$output.context.json";
for my $path ($output,$context_path) {
    $path =~ m{\Atarget/[A-Za-z0-9_./-]+\z} && $path !~ m{(?:^|/)\.\.?/} or die "output must be beneath target\n";
    !-e $path && !-l $path or die "never overwrite a plan/receipt\n";
    my $route = '.';
    for (split m{/}, $path) { $route .= "/$_"; !-l $route or die "redirected output\n"; }
}
open my $ctx, '>:raw', $context_path or die "write context: $!\n";
print {$ctx} $json->encode($context),"\n"; close $ctx or die "close context\n";
my $plan = {
    schema=>1, source=>$source, group=>$group, scope=>$narrow?'group':'workspace', policy=>$policy_path,
    inputs=>[sort(@inputs,$context_path)],
    stages=>[map { {target=>$_, required=>($narrow && exists $excluded{$_})?JSON::PP::false:JSON::PP::true,
        reason=>($narrow && exists $excluded{$_})?$excluded{$_}:'Required full-gate obligation: source/lint/native/Wasm/consumer or installed behavior remains affected or unclassified.'} } @full],
};
open my $out,'>:raw',$output or die "write plan: $!\n";
print {$out} $json->encode($plan),"\n"; close $out or die "close plan\n";
print "$output\n";
