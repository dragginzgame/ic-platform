# IC Platform browser boundary

`@tokostudios/ic-platform` owns the shared SDK Core integration boundary for
browser and module-worker consumers. Package version 0.1.0 uses exactly
`@icp-sdk/core` 6.1.1 for Blob's approved 0.48 graph. Rust package versions and
release commands remain independent; npm supplies the browser package.

Consumers declare this package instead of SDK Core. SDK Core is a regular exact
dependency, not a peer. Its cryptographic implementations, Principal/identity/
Agent classes and external Candid bytes retain their upstream identities. The
boundary does not remove the transitive SDK or promise a smaller bundle.

## Explicit imports

| Subpath | Supported responsibility |
| --- | --- |
| `/principal` | SDK Principal identity; bounded canonical text parsing with redacted errors |
| `/identity` | SDK Ed25519 identity; checked JSON key-pair restoration with a caller-supplied byte budget |
| `/identity/secp256k1` | Equivalent secp256k1 integration through the unchanged SDK |
| `/agent` | SDK HttpAgent, request-ID/CBOR operations and rejection identities; explicit Agent options and raw query dispatch |
| `/certification` | SDK Certificate verification and lookup identities; lookup alone is not authentication |
| `/candid` | SDK IDL for required external protocols |

There is no root barrel, actor generator or alternative transport/codec. Exported
SDK classes retain their public methods; these explicit names do not make every
inherited SDK method a Platform-owned implementation. Export-map changes and SDK
upgrades require consumer API/type and byte qualification.

```ts
import { parseCanonicalPrincipal } from '@tokostudios/ic-platform/principal';
import { createAgentOptions, HttpAgent, queryRaw } from '@tokostudios/ic-platform/agent';

// Application admits the origin and selects identity, trust and policy first.
const agent = await HttpAgent.create(createAgentOptions({
  host: new URL(admittedOrigin), identity, rootKey,
  retryTimes: 0, verifyQuerySignatures: true, shouldSyncTime: false,
  fetch: boundedFetch,
}));
const reply = await queryRaw(agent, parseCanonicalPrincipal(canister), {
  methodName: method, arg: wireEncodedRequest,
});
// Wire-generated bindings own payload encoding/decoding.
```

`createAgentOptions` snapshots root bytes and fixes automatic root acquisition to
false, including when called from JavaScript with an extra contrary option. It
requires HTTP(S) origin URLs, explicit identity, retry count, query
verification and construction-time synchronization choices. It performs no IO;
SDK construction may synchronize if the caller explicitly enables that choice.
Use its result with an HttpAgent subclass too, preserving Blob's intent boundary.

`queryRaw` invokes SDK query once, snapshots argument/reply bytes and preserves
upstream rejection details and transport failures. SDK-configured retries and
ancillary verification requests still apply. The function does not limit those
requests, grant update authority or install a codec. Use SDK `call`, `readState`
and `update` explicitly where needed; `update` can poll under SDK behavior. Retain
existing signed envelope bytes/request IDs and uncertain-effect reconciliation.

Trust-root and query snapshots copy only the selected byte view, including Node
Buffer inputs/replies. They never use Buffer's aliasing `slice` behavior. Invalid
query byte shapes are refused rather than coerced into an empty request; invalid
arguments are refused before invoking the SDK.

`restoreEd25519Identity(json, maxJsonBytes)` and its secp256k1 counterpart accept
the existing two-lowercase-hex-string SDK format with a 32-byte secret. The caller
selects the import budget. Reconstruction, key encoding, principal hashing and
signing remain SDK operations. A public/private mismatch is refused. Import
errors contain a code, not secret input or an upstream secret-bearing cause.
This is import integrity, not authentication, delegation admission or key custody.

Applications retain endpoint authorization, trust roots, verification/freshness
policy, timeouts, response ceilings, retries and durable effect authority. Blob
retains permission/intents/storage/worker policy. Canic retains release/schema
binding and sessions. Internet Identity login remains with its existing Auth
integration; this initial boundary covers SDK Core, not `@icp-sdk/auth`.
Owned endpoints use Wire; `/candid` is for required external Caffeine/system/
management/ledger protocols. Neither Wire nor Rust depends on this npm package.

## Qualification and migration

Prepare dependencies explicitly with `make install-browser-tools`, then run
`make check-browser`. It builds strict browser TypeScript, tests the adapters and
packs/installs an independent consumer declaring only Platform. That consumer
compiles and runs through the packed public exports, and bundles for browsers.
The Principal-only bundle must exclude Agent/identity/Candid modules.
Tool admission and packed qualification reject duplicate/aliased locked SDKs,
installed version drift and provider/Platform resolution to another Agent copy.
The package's regular LICENSE copy and packed license must match the root MIT
license. These checks do not supply the consumer-owned direct-import ban.

`make check-browser-runtime` additionally runs the packed consumer in actual
Chromium and a module worker. Prepare the selected Playwright Chromium explicitly
first with `node packages/browser/node_modules/playwright/cli.js install chromium`;
checks do not download it. Local HTTP responses are protocol fixtures,
not installed IC or production provider evidence. Keep those qualifications
separate. The current contributor environment is Node 24.21.0/npm 12.2.0; the
package declares Node >=20.19.0, but other Node versions are not qualified here.
The browser workflow selects Ubuntu 24.04 and macOS 15 Intel/Apple Silicon,
checking native host identity before setup. All three jobs passed at committed
Platform `8d80630` on 2026-10-26, including packed-provider types and actual
Chromium page/module-worker checks ([run 40293386118](https://github.com/dragginzgame/ic-platform/actions/runs/40293386118)).
This qualifies those hosts for the producer checks, not installed IC, live
provider/login behavior or downstream adoption of a published artifact.

Until maintainer publication and consumer adoption, Blob keeps its explicit SDK
peer. Afterwards replace SDK Core imports using this export map, declare Platform
as a regular dependency, and remove only the exhausted SDK Core declaration.
Keep Caffeine/Auth provider dependencies and require one compatible SDK runtime
in the final application graph. Do not bundle a private SDK copy into this library
or use bundler aliases/local patches as a supported distribution mechanism.
After adoption, consumer CI should reject first-party direct SDK Core imports and
declarations; the Rust dependency checker does not supply that JavaScript guard.
Run each consumer's retained bytes, mismatch/refusal, signed request, certificate,
worker and installed/provider acceptance before claiming adoption. Import-only
copies provide API/bundling evidence, not that full acceptance.

Caffeine 1.1.2 still declares SDK `^5.3.0`. Blob's SDK-6 application must retain
its explicit root override selecting 6.1.1; Platform's development override is
not inherited by consuming applications. Preserve Blob's reviewed Caffeine patch.
The packed provider fixture qualifies this composition, not upstream SDK-6 support.
The separate Auth type fixture uses 10.0.2, which declares SDK `^6`; Auth 8.0.3
requires SDK `^5` and is incompatible with this single-SDK graph. Canic needs a
coordinated Auth/Core migration and its own acceptance before adopting the package.

The Rust `make publish` command does not publish npm. Initial npm publication,
organization permission and release automation remain maintainer-owned work.
Source-directory npm publication runs `prepublishOnly`: tool admission, a fresh
build, adapter tests and the packed-only consumer proof must pass before upload.
`npm test` builds first; `prepack` also builds for ordinary archive preparation.
Chromium qualification remains the separate prepared-browser CI gate. The hook
adds no publication action to ordinary checks. Run it without publishing via
`npm run prepublishOnly --prefix packages/browser`.

### Initial npm supply

The implementation is committed in Rust release 0.6.17. Rust releases do not
publish the independent npm package. Initial npm supply requires explicit
publication authorization, registry authentication and package/scope write access.

From the reviewed checkout, run full `make clippy` before the longer
`make check-browser-runtime`. Then preview the actual publication path without
an upload:

```sh
npm publish ./packages/browser --dry-run --offline --access public
```

Retain the packed-consumer archive and its `1-npm.log` output: they record the
exact package version, archive integrity and file list tested by that proof.
Changes to shipped README/license files also change the archive identity; rerun
the packed proof after such edits. A successful dry-run does not establish npm
login, scope ownership or write permission.

After explicit publication authorization, the maintainer authenticates to the
public registry, verifies access to the `@dragginzgame` scope and publishes from
this source directory so `prepublishOnly` runs. Rust `make publish` is unrelated.
Read back `@tokostudios/ic-platform@0.1.0` with `npm view`, compare `dist.integrity`
with the qualified archive, and verify its sole production dependency is SDK Core
6.1.1. Report the published version/integrity to Platform #57, Blob #103 and
Canic #637. Consumers then qualify the registry artifact in their own acceptance;
only their exhausted first-party SDK declarations/imports may be removed.
