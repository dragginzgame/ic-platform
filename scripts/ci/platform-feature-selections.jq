# Input: reviewed feature names excluding default. Output: Cargo selections.
def selection: sort | join(",");
def powerset:
  reduce .[] as $feature ([[]]; . + map(. + [$feature]));
. as $features |
(if $mode == "exhaustive" then powerset | map(selection)
 elif $mode == "routine" then
   [[], $features] +
   [$features[] | [.]] +
   [range(0; $features | length) as $i |
    range($i + 1; $features | length) as $j | [$features[$i], $features[$j]]] +
   [
     # Runtime identity with redundant explicit entrypoint prerequisites.
     ["cdk", "candid", "executor", "ic0"],
     # Combined canister DTO and certificate integrations without host features.
     ["cdk", "management-canister-types", "icrc-ledger-types", "certification"],
     # Auth's portable crypto combination must not acquire host/runtime scaffolding.
     ["bls", "ecdsa", "ed25519"],
     # Certificate/signature dependency paths and portable verification together.
     ["canister-sig-creation", "signature-verification", "bls", "certification", "ed25519"],
     # Host-feature exclusion on Wasm alongside the full runtime.
     ["cdk", "agent", "candid-parser", "pocket-ic", "secp256k1", "wire-schema"],
     ($features - ["agent", "candid-parser", "pocket-ic", "secp256k1", "wire-schema"])
   ] | map(if (. - $features | length) == 0 then selection
           else error("critical selection contains an unknown feature") end)
 else error("unknown feature matrix mode") end) |
map(if . == "" then "none" else . end) + ["default"] | unique
