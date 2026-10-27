# Collapse only local implications and strictly admitted native-only leaves.
# A changed leaf contract fails closed, rather than reusing a possibly wrong graph.
def native_leaves($package):
  $policy[0].native_leaves |
  to_entries | map(. as $leaf |
    $leaf.value.prerequisites as $prerequisites |
    $leaf.value.dependencies as $dependencies |
    if ($prerequisites | type) != "array" or
       ($dependencies | type) != "array" or ($dependencies | length) == 0
    then error("malformed native-only cache policy: " + $leaf.key)
    else
      [$dependencies[] as $name |
        [$package.dependencies[] | select((.rename // .name) == $name)]] as $deps |
      if all($prerequisites[]; . as $name | $package.features[$name] != null) and
         $package.features[$leaf.key] == ($prerequisites + ($dependencies | map("dep:" + .))) and
         all($deps[]; length == 1 and .[0].optional and
             .[0].target == "cfg(not(target_family = \"wasm\"))")
      then $leaf.key else error("native-only cache contract changed: " + $leaf.key) end
    end);
def closure($features):
  . as $before |
  (. + [ .[] as $name | $features[$name][]? |
          select($features[.] != null) ]) | unique |
  if . == $before then . else closure($features) end;
($metadata[0].packages[] | select(.name == "ic-platform")) as $package |
$package.features as $features |
(if $target == "wasm32-unknown-unknown" then native_leaves($package) else [] end) as $ignored |
.[] as $selection |
(if $selection == "default" then "default"
 else (if $selection == "none" then [] else $selection | split(",") end |
       closure($features) | . - $ignored | join(",")) end) as $key |
[$selection, (if $key == "" then "none" else $key end)] | @tsv
