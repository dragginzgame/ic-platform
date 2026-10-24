# Collapse only local implications and strictly admitted native-only leaves.
# A changed leaf contract fails closed, rather than reusing a possibly wrong graph.
def native_leaves($package):
  {agent: ["ic-agent"], "candid-parser": ["candid_parser"], secp256k1: ["ic-secp256k1"], "pocket-ic": ["pocket-ic"], "wire-schema": ["ic-wire-metadata", "serde_json"]} |
  to_entries | map(. as $leaf |
    [$leaf.value[] as $name |
      [$package.dependencies[] | select((.rename // .name) == $name)]] as $deps |
    if $package.features[$leaf.key] == (["principal"] + ($leaf.value | map("dep:" + .))) and
       all($deps[]; length == 1 and .[0].optional and
           .[0].target == "cfg(not(target_family = \"wasm\"))")
    then $leaf.key else error("native-only cache contract changed: " + $leaf.key) end);
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
