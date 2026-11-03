#!/usr/bin/env perl
# A Platform pilot receipt, never a replacement for the release runner's intent.
use strict;
use warnings;
use JSON::PP;
use Digest::SHA qw(sha256_hex);
use Cwd qw(abs_path);
use File::Basename qw(dirname);
my $json=JSON::PP->new->canonical->utf8;
my ($plan_path,$run,$output)=@ARGV;
@ARGV==3 or die "usage: validation-receipt.pl PLAN RUN-DIRECTORY NEW-RECEIPT\n";
sub bytes {
    my ($path)=@_;
    $path !~ m{[\x00-\x1f]|(?:^|/)\.\.(?:/|$)} or die "unsafe evidence path\n";
    my $route=$path =~ m{^/}?'/':'.';
    for (grep {length} split m{/},$path) { $route.="/$_"; !-l $route or die "redirected evidence\n"; }
    -f $path or die "missing evidence $path\n";
    open my $fh,'<:raw',$path or die "read $path: $!\n";
    local $/; return <$fh>;
}
sub command {
    open my $fh,'-|',@_ or die "execute @_: $!\n";
    local $/; my $value=<$fh>//''; close $fh or die "failed @_; no receipt\n"; return $value;
}
my $raw=bytes($plan_path);
my $plan=$json->decode($raw);
$raw eq bytes("$run/parsed-inputs.json.plan.json") or die "runner selected a different plan\n";
bytes($plan->{policy}) eq bytes("$run/parsed-inputs.json.policy.json") or die "policy drift\n";
command('git','rev-parse','HEAD') eq "$plan->{source}\n" or die "stale source\n";
command('git','status','--porcelain','--untracked-files=all') eq '' or die "dirty release inputs\n";
my $context=$json->decode(bytes("$plan_path.context.json"));
my $runner_log=dirname(dirname($run)).'/runner.log';
my $runner_bytes=bytes($runner_log);
$runner_bytes =~ /^VALIDATION PASSED: all requested targets succeeded\.$/m
    && $runner_bytes =~ /\nPlatform runner completed status=0\n\z/
    or die "runner did not finish successfully\n";
command('git','ls-files','--stage','-z') eq $context->{index_inventory} or die "source inventory changed\n";
$context->{source} eq $plan->{source} && $context->{group} eq $plan->{group} or die "wrong context\n";
command('git','rev-parse',"refs/tags/$context->{baseline_tag}^{commit}") eq "$context->{baseline}\n"
    or die "baseline moved\n";
# Reuse Shared's checkpoint validator; do not implement a second source hasher.
my $observed="$run/receipt-inputs.json";
system('perl','scripts/ci/validation-input-checkpoint.pl',abs_path('.'),"$run/plan-inputs.json",$observed,
    'scripts/ci/verify-file-checksum.sh',@{$plan->{inputs}})==0 or die "input checkpoint drift\n";
my %last;
for my $line (split /\n/,bytes("$run/plan-events.tsv")) {
    my ($target,$state,$detail)=split /\t/,$line,3;
    next if $target eq 'target';
    $last{$target}=[$state,$detail];
}
my %timings;
for my $line (split /\n/,bytes("$run/timings.tsv")) {
    my ($target,$state,$seconds,$log)=split /\t/,$line,4;
    next if $target eq 'target';
    !exists $timings{$target} && $state eq 'PASS' && $seconds =~ /^\d+$/
        or die "failed/duplicate/incomplete stage timing\n";
    $timings{$target}=[$seconds,$log];
}
my @stages;
for my $stage (@{$plan->{stages}}) {
    my $target=$stage->{target}; my $event=$last{$target} or die "missing stage event\n";
    if ($stage->{required}) {
        $event->[0] eq 'PASS' && exists $timings{$target} && $event->[1] eq $timings{$target}[1]
            or die "blocked, failed or missing required stage $target\n";
        my $log=$event->[1];
        index(abs_path($log)//'',abs_path($run).'/')==0 or die "stage log outside attempt\n";
        push @stages,{target=>$target,state=>'PASS',admission=>'executed',seconds=>0+$timings{$target}[0],
            log=>$log,log_sha256=>sha256_hex(bytes($log))};
    } else {
        $event->[0] eq 'not-applicable' && !exists $timings{$target} or die "excluded stage executed\n";
        push @stages,{target=>$target,state=>'not-applicable',reason=>$stage->{reason}};
    }
    delete $last{$target}; delete $timings{$target};
}
!keys(%last) && !keys(%timings) or die "unknown evidence stage\n";
my %environment=map {$_=>(exists($ENV{$_})?$ENV{$_}:undef)} keys %{$context->{environment}};
$json->encode(\%environment) eq $json->encode($context->{environment}) or die "configuration drift\n";
my ($host,$rustc,$cargo)=(command('uname','-sm'),command('rustc','-vV'),command('cargo','-V'));
$host eq $context->{host} && $rustc eq $context->{rustc} && $cargo eq $context->{cargo}
    or die "host/compiler drift\n";
my $receipt={schema=>1,contract=>'platform-release-validation-pilot/1',source=>$plan->{source},group=>$plan->{group},
    scope=>$plan->{scope},baseline=>$context->{baseline},plan_sha256=>sha256_hex($raw),
    policy_sha256=>sha256_hex(bytes($plan->{policy})),inputs_sha256=>sha256_hex(bytes("$run/plan-inputs.json")),
    host=>$host,rustc=>$rustc,cargo=>$cargo,runner_log_sha256=>sha256_hex(bytes($runner_log)),
    environment=>\%environment,stages=>\@stages,release_omission_admitted=>JSON::PP::false};
!-e $output && !-l $output or die "never overwrite a receipt\n";
open my $out,'>:raw',$output or die "write receipt: $!\n";
print {$out} $json->encode($receipt),"\n"; close $out or die "close receipt\n";
print "$output\n";
