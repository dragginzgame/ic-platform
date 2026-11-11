# Resolve actual normal dependency edges, not every package present in a lock.
def require($ok; $message): if $ok then . else error($message) end;
def package($id): .packages[] | select(.id == $id);
def edges($id): [.resolve.nodes[] | select(.id == $id) | .deps[]
    | select(any(.dep_kinds[]; .kind == null)) | .pkg];
def dep($id; $name): . as $m | [edges($id)[] | . as $p | $m | package($p) | select(.name == $name)]
    | if length == 1 then .[0] else error("expected one " + $name + " edge from " + $id) end;
def owner: .name | test("^ic-platform(-types|-runtime|-macros|-runtime-guard)?$");
def owners($id): . as $m | [edges($id)[] | . as $p | $m | package($p) | select(owner)] as $direct
    | [$direct[], ($direct[] | .id as $p | $m | owners($p)[])] | unique_by(.id);
def identity: {name, version, id, source, manifest_path};
. as $m |
[.packages[] | select(.name == $fixture)] as $roots |
require(($roots | length) == 1; "fixture package missing or ambiguous: " + $fixture) |
$roots[0] as $root |
owners($root.id) as $selected |
require(all($required | split(",")[]; . as $name | any($selected[]; .name == $name) or
    any($m | edges($root.id)[] | . as $id | $m | package($id); .name == $name)); "required qualification dependency missing") |
require(all($selected | group_by(.name)[]; length == 1); "duplicate selected public owner") |
require(all($selected[];
    if $mode == "source" then .source == null and .manifest_path == ($source_root + "/crates/" + .name + "/Cargo.toml")
    else .source == "registry+https://github.com/rust-lang/crates.io-index" end);
    "selected owner is not the requested source") |
require(($mode != "published" or all($selected[] | select(.name | test("^ic-platform-(runtime|macros|runtime-guard)$")); .version == $runtime_version)); "wrong published Runtime cohort") |
[edges($root.id)[] | . as $p | $m | package($p) | select(.name == "ic-testkit")] as $testkits |
require(($testkits | length) <= 1; "ambiguous Testkit infrastructure") |
[ $testkits[] | . as $t |
    {testkit: ($t | identity), platform_owners: ($m | owners($t.id) | map(identity)),
     coverage: "Testkit infrastructure; not qualification of the selected Platform implementation"}
] as $infrastructure |
# Agent passes the re-exported upstream PocketIC CanisterId into Agent APIs.
# Pin that actual re-export chain's package identity; unrelated lock owners are allowed.
[ $selected[] | select(.name == "ic-platform") | .id as $p |
    $m | edges($p)[] | . as $id | $m | package($id) | select(.name == "ic-agent")
] as $agents |
[ $agents[] | . as $agent | $testkits[] | . as $t |
    ($m | dep($agent.id; "ic_principal")) as $agent_principal |
    ($m | dep($t.id; "pocket-ic")) as $client |
    ($m | dep($client.id; "candid")) as $candid |
    ($m | dep($candid.id; "ic_principal")) as $testkit_principal |
    require($agent_principal.id == $testkit_principal.id; "Agent/Testkit CanisterId identity mismatch") |
    {boundary: "Testkit CanisterId -> Agent", principal: ($agent_principal | identity)}
] as $crossings |
{schema: 1, fixture: ($root | identity), selection: $mode,
 selected_owners: ($selected | map(identity)), infrastructure: $infrastructure,
 client_suppliers: ($agents | map(identity)),
 public_type_crossings: $crossings}
