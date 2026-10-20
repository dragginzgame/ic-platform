use strict; use warnings; use JSON::PP;
my $root=$ENV{BENCH_ROOT} or die 'Set BENCH_ROOT';
open my $input, '<', "$root/results/results.tsv" or die $!;
<$input>; my %rows;
while (<$input>) { chomp; my @r=split /\t/; push @{$rows{$r[0]}{$r[1]}}, \@r; }
my %summary;
for my $case (sort keys %rows) {
 for my $revision (qw(base candidate)) {
  my @rows=@{$rows{$case}{$revision}};
  my @times=sort {$a<=>$b} map {$_->[3]+0} @rows;
  die 'incomplete case' unless @times==3;
  for my $col (5,6,7,8,9) {die 'nonreproducible size/graph' if grep {$_->[$col] ne $rows[0][$col]} @rows;}
  $summary{builds}{$case}{$revision}={median_seconds=>$times[1], min_seconds=>$times[0], max_seconds=>$times[-1], wasm_bytes=>0+$rows[0][5], code_section_bytes=>0+$rows[0][6], custom_section_bytes=>0+$rows[0][7], gzip_bytes=>0+$rows[0][8], packages=>0+$rows[0][9]};
 }
}
open my $principal,'<',"$root/results/principal.tsv" or die $!;
<$principal>; my %samples;
while (<$principal>) {chomp; my ($rev,$run,$len,$op,$count,$ns)=split /\t/; push @{$samples{$len}{$op}{$rev}}, $ns/$count;}
for my $len (keys %samples) {for my $op (keys %{$samples{$len}}) {for my $rev (keys %{$samples{$len}{$op}}) {my @v=sort {$a<=>$b} @{$samples{$len}{$op}{$rev}}; die 'incomplete principal samples' unless @v==5; $summary{principal}{$len}{$op}{$rev}={median_ns=>$v[2],min_ns=>$v[0],max_ns=>$v[-1]};}}}
$summary{method}={base=>'390644f',candidate=>'9a661f7',principal_base=>'f105386',rust=>'1.99.1',build_jobs=>4,build_runs=>3,principal_runs=>5,principal_iterations=>500000,build_cache=>'fresh target directory, cached registry, offline; serial, alternating revision order',profile=>'release opt-level=3, lto=false, codegen-units=16, panic=abort; Wasm strip=debuginfo',limitations=>'Single Linux machine. Native Principal timings are not IC instruction measurements. Runtime fixture includes registry Platform 0.1.18 through its historical Timers/Wire dependencies. Signature probes are exported qualification functions, not complete deployed Auth applications.'};
print JSON::PP->new->pretty->canonical->encode(\%summary);
