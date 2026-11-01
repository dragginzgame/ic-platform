# New Runtime ownership; retain the legacy predicate below for historical archives.
if any(.packages[]; .name == "ic-platform-runtime") or ($ARGS.named.runtime_owner // false) then
    . as $metadata |
    [.packages[] | select(.name == "ic-platform-runtime")] as $runtimes |
    [.packages[] | select(.name == "ic-platform")] as $facades |
    [.packages[] | select(.name == "ic-platform-macros")] as $macros |
    [.packages[] | select(.name == "ic-platform-runtime-guard")] as $guards |
    ($runtimes + $facades + $macros + $guards) as $owners |
    ($runtimes | length) == 1 and
    all($owners[]; .source == "registry+https://github.com/rust-lang/crates.io-index") and
    ($owners | map([.name,.version]) | unique | length) == ($owners | length) and
    (if ($ARGS.named.runtime_owner // false) then $runtimes[0].version == $version
     else ($facades | length) == 1 and $facades[0].version == $version end) and
    ($guards | length) <= 1 and all($guards[]; .version == $runtimes[0].version) and
    (if $require_generator or any(($metadata.resolve.nodes // [])[];
        .id == $runtimes[0].id and (.features | index("executor")) != null)
     then ($guards | length) == 1 else true end) and
    ($macros | length) <= 1 and all($macros[]; .version == $runtimes[0].version) and
    (if $require_generator then ($macros | length) == 1 else true end) and
    # No legacy embedded executor may accompany the new Runtime owner.
    all($facades[]; . as $facade |
        if any(($facade.dependencies // [])[]; .name == "ic-platform-runtime-guard") and
           any(($metadata.resolve.nodes // [])[]; .id == $facade.id and (.features | index("executor")) != null)
        then false else true end)
else
# All boundary packages must be registry identities. Runtime fixtures also have
# transitive facades through Timers/Testkit: preserve that graph while requiring
# the exact supplier/generator pair. Standalone proofs retain one facade.
[.packages[] | select(.name == "ic-platform")] as $facades |
[.packages[] | select(.name == "ic-platform-macros")] as $generators |
[.packages[] | select(.name == "ic-platform-runtime-guard")] as $guards |
($facades + $generators + $guards) as $packages |
# Historical facades had no helper; current executor selections must resolve it.
. as $metadata |
all($facades[]; . as $facade |
    if any(($facade.dependencies // [])[]; .name == "ic-platform-runtime-guard") and
       any(($metadata.resolve.nodes // [])[]; .id == $facade.id and (.features | index("executor")) != null)
    then any($guards[]; .version == $facade.version) else true end) and
($guards | length) <= 1 and all($guards[]; .version == $version) and
($require_generator | type) == "boolean" and
all($packages[]; .source == "registry+https://github.com/rust-lang/crates.io-index") and
($packages | map([.name, .version]) | unique | length) == ($packages | length) and
([$facades[] | select(.version == $version)] | length) == 1 and
if $require_generator then
    ([$generators[] | select(.version == $version)] | length) == 1
else
    ($facades | length) == 1 and ($generators | length) <= 1 and
    all($generators[]; .version == $version)
end

end
