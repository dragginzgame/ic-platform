# Shared package classifications. Each caller still observes and admits its own graph.
function platform_host_capability(name) {
    if (name == "candid_parser") return "candid-parser"
    if (name == "ic-agent") return "agent"
    if (name == "pocket-ic" || name == "ic-platform-pocket-ic") return "pocket-ic"
    if (name == "ic-secp256k1") return "secp256k1"
    if (name == "ic-testkit" || name == "ic-testkit-candid") return "testkit"
    if (name == "ic-management-canister-types") return "management-native"
    return ""
}
function platform_forbidden(family, name) {
    if (family == "minimal")
        return name ~ /^(ic_principal|serde|serde_core|serde_derive|sha2|digest|crc32fast|data-encoding|candid|ic-agent)$/
    if (family == "identity")
        return name ~ /^(ic-platform-runtime|icrc-ledger-types|icrc-cbor|minicbor|base32|candid_parser|ic-agent|pocket-ic|ic-platform-pocket-ic)$/
    if (family == "runtime-host")
        return platform_host_capability(name) != "" && !(allow_native_management && name == "ic-management-canister-types")
    if (family == "retired-cdk")
        return name ~ /^ic-cdk/ || (name == "ic0" && !allow_signature_ic0)
    if (family == "external-codecs")
        return name ~ /^(ic-cdk[^ ]*|ic0|ic-platform-macros|ic-platform-runtime|ic-platform-runtime-guard|ic-agent|pocket-ic|ic-platform-pocket-ic|ic-testkit|candid_parser|reqwest|tokio)$/
    return 0
}
# Standalone use reports whether the graph contains a forbidden package.
family != "" { if (platform_forbidden(family, $1)) forbidden_package = 1 }
runtime_mode != "" {
    if ($1 == "candid") runtime_candid = 1
    if ($1 == "candid" || $1 == "candid_derive") runtime_codec = 1
    if (platform_forbidden("runtime-host", $1)) runtime_host = 1
    if ($0 ~ /^ic-canister-sig-creation v1\.3\.1( \(\*\))?$/) runtime_signature = 1
    if ($1 == "ic0" && $0 !~ /^ic0 v1\.2\.0( \(\*\))?$/) runtime_bad_ic0 = 1
    if ($1 ~ /^ic-cdk/ || $1 == "ic0") runtime_retired = 1
    if ($1 ~ /^ic-cdk/) runtime_cdk = 1
    if ($1 == "ic-platform-runtime") {
        identity = $0
        sub(/ \(\*\)$/, "", identity)
        if (!(identity in runtime_identities)) { runtime_identities[identity] = 1; runtime_count++ }
    }
}
function runtime_reject(message) {
    print message ": " FILENAME > "/dev/stderr"
    exit 1
}
END {
    if (family != "") exit !forbidden_package
    if (runtime_mode != "") {
        if (runtime_mode == "wire" && runtime_codec) runtime_reject("Wire runtime acquired Candid")
        if (runtime_mode == "external" && !runtime_candid) runtime_reject("Missing selected external Candid adapter")
        if (runtime_host) runtime_reject("Canister graph contains host parsing, Agent or Testkit")
        if (runtime_signature && runtime_bad_ic0) runtime_reject("Unqualified signature IC0 supplier")
        if (runtime_retired && (!runtime_signature || runtime_cdk)) runtime_reject("Retired full CDK runtime/generator")
        if (runtime_count != 1) runtime_reject("Runtime must have one canonical Runtime executor owner")
    }
}
