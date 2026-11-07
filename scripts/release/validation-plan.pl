#!/usr/bin/env perl
# Consumer-owned full-gate roster; execution/checkpointing belongs to Shared.
use strict;
use warnings;
use JSON::PP;
use File::Path qw(make_path);
use File::Basename qw(dirname);
my $json = JSON::PP->new->canonical->utf8;
my ($group, $source, $baseline, $output, $scope) = @ARGV;
@ARGV == 5 && $scope =~ /\A(?:qualification|release)\z/
    or die "usage: validation-plan.pl GROUP SOURCE GROUP-TAG|none OUTPUT qualification|release\n";
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
my $execution = $json->decode(capture('perl','scripts/release/validation-context.pl'));
my $release;
if ($scope eq 'release') {
    $release = {map {$_ => $ENV{"RELEASE_".uc($_)}} qw(previous version remote branch)};
    for (qw(previous version)) {
        defined($release->{$_}) && $release->{$_} =~ /\A\d+\.\d+\.\d+\z/ or die "missing release version identity\n";
    }
    for (qw(remote branch)) { defined($release->{$_}) && length($release->{$_}) or die "missing release destination\n"; }
    $baseline eq 'none' || $baseline eq "$group-v$release->{previous}" or die "wrong release baseline\n";
}
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
make_path(dirname($output));
open my $ctx, '>:raw', $context_path or die "write context: $!\n";
print {$ctx} $json->encode($context),"\n"; close $ctx or die "close context\n";
my $plan = {
    schema=>1, source=>$source, group=>$group, scope=>'workspace', policy=>$policy_path,
    inputs=>[sort(@inputs,$context_path)],
    stages=>[map { {target=>$_, required=>JSON::PP::true,
        reason=>'Every Platform release requires the complete local gate.'} } @full],
};
open my $out,'>:raw',$output or die "write plan: $!\n";
print {$out} $json->encode($plan),"\n"; close $out or die "close plan\n";
print "$output\n";
