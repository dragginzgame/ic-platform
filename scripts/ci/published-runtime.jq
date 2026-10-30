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
