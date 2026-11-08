# Validate every selection against its cached graph in one process per target.
# Input TSV: original selection, target-aware cache key. Graphs remain Cargo output.
BEGIN {
    FS = "\t"
    if (activation == "") { print "missing feature activation input" > "/dev/stderr"; exit 1 }
    while ((status = (getline < activation)) > 0) {
        if (NF != 2 || $1 == "" || $2 == "" || ($1 in active_features)) {
            print "invalid feature activation input" > "/dev/stderr"; exit 1
        }
        active_features[$1] = $2
    }
    close(activation)
    if (status < 0) { print "unreadable feature activation input" > "/dev/stderr"; exit 1 }
}
function enabled(name) { return index("," active ",", "," name ",") != 0 }
function reject(reason) {
    print target ": " selection ": " reason " (" graph ")" > "/dev/stderr"
    exit 1
}
{
    if (NF != 2 || $1 == "" || $2 == "") reject("invalid selection mapping")
    selection = $1
    key = $2
    if (!(selection in active_features)) reject("missing feature activation")
    active = active_features[selection]
    graph = cache "/" target "-" key ".txt"
    if (!(key in loaded)) {
        while ((status = (getline line < graph)) > 0) {
            # Only retain policy-relevant lines; ordinary transitive nodes cannot
            # affect admission. Keep complete lines for source/identity checks.
            split(line, fields, " ")
            if (fields[1] ~ /^(ic|candid|pocket-ic|ed25519-dalek|curve25519-dalek|k256|blake3)/ ||
                fields[1] ~ /^(rand|rand_chacha|getrandom|pretty|serde|serde_core|serde_derive|sha2|digest|crc32fast|data-encoding|serde_cbor|half)$/)
                lines[key, ++count[key]] = line
            nonempty = 1
        }
        close(graph)
        if (status < 0 || !nonempty) reject("missing or empty graph")
        nonempty = 0
        loaded[key] = 1
    }
    native = target !~ /^wasm/
    parser = native && enabled("candid-parser")
    agent = native && enabled("agent")
    protocol = native && enabled("ic-protocol")
    external_endpoint = enabled("candid-endpoint")
    generator_enabled = enabled("candid-endpoint")
    runtime = enabled("runtime")
    executor = enabled("executor")
    verification = enabled("signature-verification")
    creation = enabled("canister-sig-creation")
    bls = verification || agent || enabled("bls")
    raw_system = executor || enabled("ic0")
    certification = creation || agent || enabled("certification")
    management = enabled("management-canister-types")
    ledger = enabled("icrc-ledger-types")
    external_dto = enabled("cmc-types") || enabled("icp-ledger-types") || enabled("cycles-ledger-types") || enabled("nns-registry-types") || enabled("certified-assets-types")
    candid = external_endpoint || enabled("candid") || enabled("candid-service") || parser || agent || protocol || management || ledger || external_dto || creation
    external_principal = enabled("principal-external")
    principal = enabled("principal")
    hashing = enabled("principal-self-authenticating") || creation || candid || certification || bls || enabled("ecdsa") || enabled("ed25519")
    minimal = !principal && !enabled("bls") && !enabled("ecdsa") && !enabled("ed25519") && !enabled("certification")
    found_ed = 0
    found_curve = 0
    found_k256 = 0
    found_guard = 0
    for (i = 1; i <= count[key]; i++) {
        line = lines[key, i]
        n = split(line, f, " ")
        name = f[1]; version = f[2]
        host_capability = platform_host_capability(name)
        if (minimal && platform_forbidden("minimal", name))
            reject("application types in minimal executor/IC0 graph")
        if (platform_forbidden("external-principal", name) && !external_principal)
            reject("external Principal in owned-only graph")
        if (platform_forbidden("hashing", name) && !hashing)
            reject("unselected hashing")
        if (name == "ic-representation-independent-hash" && !(agent || creation))
            reject("unselected signature hashing")
        if ((name == "serde_cbor" || name == "half") && !(creation || verification || agent || parser || enabled("certification-cbor")))
            reject("unselected CBOR parser")
        if (name == "pretty" && !(enabled("candid-service") || parser || agent || protocol || creation || (native && management)))
            reject("unselected Candid service printer")
        if (name == "ic-transport-types" && !(agent || protocol)) reject("unselected native IC transport types")
        registry = n == 2 || (n == 3 && f[3] == "(*)")
        if (name == "ic-host-artifacts") reject("host artifact codecs belong to Tooling consumers")
        if (name == "blake3") reject("pure hashing belongs directly to consumers")
        # rand_core supplies traits and is permitted. Entropy/RNG implementations
        # belong only to the selected native host capabilities that require them.
        if (name ~ /^(rand|rand_chacha|getrandom)$/ && !(agent || parser))
            reject("unselected RNG/entropy scaffolding")
        if (name == "k256") {
            if (!(enabled("ecdsa") || agent) || !registry) reject("unselected K256")
            found_k256 = 1
        }
        if (name == "ed25519-dalek" || name == "curve25519-dalek") {
            if (!(enabled("ed25519") || agent) || !registry) reject("unselected Dalek")
            if (name == "ed25519-dalek") found_ed = 1
            else found_curve = 1
        }
        if (host_capability == "secp256k1") reject("offline public derivation is retired")
        if (host_capability == "pocket-ic") reject("PocketIC belongs to Testkit, outside Platform")
        if (name == "pocket-ic") reject("retired registry PocketIC client")
        if (host_capability == "testkit") reject("Testkit in production")
        if (name ~ /^(ic-wire|ic-wire-derive)$/) reject("Wire in production")
        if (host_capability == "candid-parser" && !parser) reject("unselected host parser")
        if (host_capability == "agent" && !agent) reject("unselected host Agent")
        if (platform_forbidden("retired-cdk", name) && !(name == "ic0" && creation && registry && version == "v1.2.0")) reject("external CDK/executor/IC0 supplier is retired")
        if (name == "ic-platform-runtime" && !raw_system) reject("unselected runtime owner")
        if (name == "ic-platform-runtime-guard") {
            if (!executor || (line != guard && line != guard " (*)")) reject("unqualified runtime singleton guard")
            found_guard = 1
        }
        if (name == "ic-platform-macros" && (!generator_enabled || (line != generator && line != generator " (*)"))) reject("unqualified generator")
        if (name == "ic-canister-sig-creation" && (!creation || !registry || version != "v1.3.1")) reject("unqualified signature key dependency")
        if (name == "ic-signature-verification" && (!verification || !registry || version != "v0.3.0")) reject("unqualified signature verifier")
        if (name == "ic-verify-bls-signature" && (!bls || version != "v0.6.0" || !registry)) reject("unqualified upstream BLS verifier")
        if (name == "ic_bls12_381" && (!bls || version != "v0.10.1" || !registry)) reject("unqualified upstream BLS curve")
        if (name == "ic-certification" && (!certification || version != "v3.2.0" || !registry)) reject("unqualified certification")
        if (name == "ic-management-canister-types" && !(registry && native && (management) && version == "v0.11.0")) reject("unselected or Wasm registry management types")
        if (name == "icrc-ledger-types" || name == "icrc-cbor") reject("retired upstream ledger package")
        if (name == "ic-error-types" && !(agent)) reject("runtime error catalogue")
        if (name == "ic-stable-structures") reject("retired storage substrate")
        if ((name == "candid" || name == "candid_derive") && !candid) reject("unexpected Candid")
    }
    if (executor && !found_guard) reject("missing runtime singleton guard")
    if (enabled("ed25519") && (!found_ed || !found_curve)) reject("missing selected Dalek")
    if (enabled("ecdsa") && !found_k256) reject("missing selected K256")
}
