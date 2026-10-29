# Validate the actual package once; messages remain local contract diagnostics.
def require(condition; $message):
  [condition] as $results |
  if ($results | length) > 0 and all($results[]; . == true)
  then . else error($message) end;
[.packages[] | select(.name == "ic-platform")] |
if length == 1 then .[0] else error("expected exactly one ic-platform package") end |
require((.features.candid == ["principal", "dep:candid", "candid/bignum"] and
    .features["candid-service"] == ["candid", "candid/printer"]); "Candid service generation must remain explicitly selected") |
require(((.features | keys) == $policy[0].features); "Platform feature inventory changed; update the graph matrix and isolation policy") |
require((.features["candid-endpoint"] == ["cdk", "candid"]); "External Candid endpoints must explicitly select the existing generator and Candid codec") |
require((.features.default == []); "Platform must declare empty default features") |
require((.dependencies[] | select(.name == "ic-cdk-executor") | .req == "=2.0.0"); "Platform must preserve the exact qualified executor =2.0.0") |
require((. as $p | .features.cdk == ["runtime", "dep:ic-platform-macros"] and .features.runtime == ["principal", "executor", "ic0", "dep:pin-project-lite", "dep:thiserror"] and ([.dependencies[] | select(.name == "ic-cdk")] | length == 0) and ([.dependencies[] | select(.name == "ic-platform-macros")] | length == 1 and .[0].optional and .[0].req == ("^" + $p.version))); "CDK must select only the owned adapters, qualified executor/ic0 and coordinated generator") |
require((.features.secp256k1 == ["principal", "dep:ic-secp256k1"] and ([.dependencies[] | select(.name == "ic-secp256k1")] | length == 1 and .[0].optional and .[0].req == "=0.3.0" and .[0].target == "cfg(not(target_family = \"wasm\"))")); "secp256k1 must remain exact, optional and native-only") |
require((.dependencies[] | select(.name == "ic0") | .req == "=1.2.0"); "Platform must preserve the exact qualified ic0 =1.2.0") |
require((.features["management-canister-types"] == ["principal", "dep:ic-management-canister-types"] and ([.dependencies[] | select(.name == "ic-management-canister-types")] | length == 1 and .[0].req == "=0.11.0" and .[0].optional)); "Platform must preserve optional management types =0.11.0") |
require((.features["icrc-ledger-types"] == ["principal", "dep:icrc-ledger-types", "dep:candid"] and ([.dependencies[] | select(.name == "icrc-ledger-types")] | length == 1 and .[0].optional and (.[0].uses_default_features | not) and .[0].features == [])); "Ledger types must be optional without upstream default/storage features; ICRC-10 uses the existing Candid identity") |
require((.features.bls == ["dep:ic-verify-bls-signature"] and ([.dependencies[] | select(.name == "ic-verify-bls-signature")] | length == 1 and .[0].optional and (.[0].uses_default_features | not) and .[0].features == ["alloc"])); "BLS must be optional with alloc and no upstream default/RNG features") |
require((.features.ed25519 == ["dep:curve25519-dalek", "dep:ed25519-dalek"] and ([.dependencies[] | select(.name == "curve25519-dalek" or .name == "ed25519-dalek")] | length == 2 and all(.[]; .optional and (.uses_default_features | not) and .features == [] and .target == null))); "Ed25519 must remain optional on native/Wasm without upstream default/RNG features") |
require((.features["stable-structures"] == ["dep:ic-stable-structures"] and ([.dependencies[] | select(.name == "ic-stable-structures")] | length == 1 and .[0].optional and .[0].req == "=0.7.2" and (.[0].uses_default_features | not) and .[0].features == [] and .[0].target == null)); "Storage substrate must remain optional, portable and aligned with Memory =0.7.2") |
require((.features.ecdsa == ["dep:k256"] and ([.dependencies[] | select(.name == "k256")] | length == 1 and .[0].optional and (.[0].uses_default_features | not) and .[0].features == ["ecdsa"] and .[0].target == null)); "ECDSA must remain optional on native/Wasm without default/RNG features") |
true
