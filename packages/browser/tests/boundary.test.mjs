import assert from 'node:assert/strict';
import test from 'node:test';
import { Principal as SdkPrincipal } from '@icp-sdk/core/principal';
import { Ed25519KeyIdentity as SdkEd25519 } from '@icp-sdk/core/identity';
import { Secp256k1KeyIdentity as SdkSecp256k1 } from '@icp-sdk/core/identity/secp256k1';
import {
  HttpAgent as SdkAgent, Certificate as SdkCertificate, Endpoint as SdkEndpoint,
  SubmitRequestType as SdkSubmitRequestType, ReadRequestType as SdkReadRequestType, Expiry as SdkExpiry,
} from '@icp-sdk/core/agent';
import { Principal, parseCanonicalPrincipal, PrincipalTextError } from '../dist/principal.js';
import { Ed25519KeyIdentity, restoreEd25519Identity, IdentityImportError } from '../dist/identity.js';
import { Secp256k1KeyIdentity, restoreSecp256k1Identity } from '../dist/secp256k1.js';
import { HttpAgent, createAgentOptions, queryRaw, Cbor, requestIdOf, RejectError, IC_ROOT_KEY } from '../dist/agent.js';
import { Endpoint, SubmitRequestType, ReadRequestType, Expiry } from '../dist/agent.js';
import { Certificate } from '../dist/certification.js';
import { IDL } from '../dist/candid.js';

test('external identities stay exact; canonical Principal parsing is bounded', () => {
  assert.equal(Principal, SdkPrincipal);
  assert.equal(HttpAgent, SdkAgent);
  assert.equal(Certificate, SdkCertificate);
  for (let length = 0; length <= 29; length++) {
    const bytes = new Uint8Array(length).fill(2);
    const text = SdkPrincipal.fromUint8Array(bytes).toText();
    assert.deepEqual(parseCanonicalPrincipal(text).toUint8Array(), bytes);
  }
  for (const text of [null, 42, '', 'invalid', '2VXSX-FAE', 'a'.repeat(64)]) {
    assert.throws(() => parseCanonicalPrincipal(text), PrincipalTextError);
  }
});

test('identity restoration retains SDK keys/signatures and refuses mismatched pairs', async () => {
  assert.equal(Ed25519KeyIdentity, SdkEd25519);
  assert.equal(Secp256k1KeyIdentity, SdkSecp256k1);
  for (const [sdk, restore] of [[SdkEd25519, restoreEd25519Identity], [SdkSecp256k1, restoreSecp256k1Identity]]) {
    const original = sdk.fromSecretKey(new Uint8Array(32).fill(7));
    const json = JSON.stringify(original.toJSON());
    const restored = restore(json, 512);
    assert.equal(restored.getPrincipal().toText(), original.getPrincipal().toText());
    assert.deepEqual(await restored.sign(Uint8Array.of(1, 2)), await original.sign(Uint8Array.of(1, 2)));
    const wrong = original.toJSON();
    wrong[1] = sdk.fromSecretKey(new Uint8Array(32).fill(8)).toJSON()[1];
    assert.throws(() => restore(JSON.stringify(wrong), 512), { code: 'key-mismatch' });
    for (const [input, budget, code] of [[json, 1, 'budget'], [json, 0, 'budget'], ['{', 512, 'json'], ['["00","00"]', 512, 'key']]) {
      assert.throws(() => restore(input, budget), error => {
        assert(error instanceof IdentityImportError);
        assert.equal(error.code, code);
        assert.equal(error.cause, undefined);
        assert(!error.message.includes(json));
        return true;
      });
    }
  }
});

test('signing requests retain SDK endpoint, expiry and signed-envelope identities', async () => {
  assert.equal(Endpoint, SdkEndpoint);
  assert.equal(SubmitRequestType, SdkSubmitRequestType);
  assert.equal(ReadRequestType, SdkReadRequestType);
  assert.equal(Expiry, SdkExpiry);
  const json = '{"__expiry__":"100000000000"}';
  const expiry = Expiry.fromJSON(json);
  assert.equal(expiry.toBigInt(), 100000000000n);
  assert.equal(JSON.stringify(expiry), json);
  assert.deepEqual(expiry.toHash(), SdkExpiry.fromJSON(json).toHash());
  const identity = Ed25519KeyIdentity.fromSecretKey(new Uint8Array(32).fill(7));
  const sdkIdentity = SdkEd25519.fromSecretKey(new Uint8Array(32).fill(7));
  const sender = identity.getPrincipal();
  const canister_id = Principal.anonymous();
  const ingress_expiry = expiry;
  const method = { canister_id, method_name: 'wire_method', arg: Uint8Array.of(1, 2) };
  for (const [endpoint, body] of [
    [Endpoint.Call, { ...method, request_type: SubmitRequestType.Call, sender, ingress_expiry }],
    [Endpoint.Query, { ...method, request_type: ReadRequestType.Query, sender, ingress_expiry }],
    [Endpoint.ReadState, { request_type: ReadRequestType.ReadState, paths: [[Uint8Array.of(1)]], sender, ingress_expiry }],
  ]) {
    const request = { endpoint, request: { method: 'POST' }, body };
    const original = Cbor.encode(body);
    const signed = await identity.transformRequest(request);
    assert.deepEqual(signed, await sdkIdentity.transformRequest(request));
    assert.deepEqual(signed.body.content, body);
    assert.deepEqual(Cbor.encode(request.body), original);
  }
});

function options(overrides = {}) {
  return {
    host: new URL('https://icp-api.io'),
    rootKey: Uint8Array.from(Buffer.from(IC_ROOT_KEY, 'hex')),
    identity: Ed25519KeyIdentity.fromSecretKey(new Uint8Array(32).fill(7)),
    retryTimes: 0, verifyQuerySignatures: false, shouldSyncTime: false,
    ...overrides,
  };
}

test('explicit Agent construction owns root bytes and performs no discovery IO', async () => {
  let calls = 0;
  const input = options({ fetch: async () => { calls++; throw new Error('unexpected IO'); } });
  const result = createAgentOptions({ ...input, shouldFetchRootKey: true });
  const root = result.rootKey.slice();
  input.rootKey.fill(0);
  input.host.hostname = 'changed.invalid';
  assert.deepEqual(result.rootKey, root);
  assert.equal(result.host, 'https://icp-api.io');
  assert.equal(result.shouldFetchRootKey, false);
  await HttpAgent.create(result);
  assert.equal(calls, 0);
  for (const override of [{ retryTimes: undefined }, { retryTimes: -1 }, { verifyQuerySignatures: undefined },
    { shouldSyncTime: undefined }, { identity: undefined }, { rootKey: new Uint8Array() },
    { host: new URL('https://user:secret@example.test/') }, { host: new URL('https://example.test/path') }]) {
    assert.throws(() => createAgentOptions(options(override)), TypeError);
  }
});

test('trust-root snapshots own only the selected Node Buffer bytes', () => {
  const backing = Buffer.from([99, 1, 2, 3, 99]);
  const rootKey = backing.subarray(1, 4);
  const result = createAgentOptions(options({ rootKey }));
  backing.fill(0);
  assert.deepEqual([...result.rootKey], [1, 2, 3]);
  assert(!Buffer.isBuffer(result.rootKey));
});

test('query arguments own selected Buffer bytes across SDK awaits', async () => {
  const backing = Buffer.from([99, 5, 6, 7, 99]);
  const effectiveCanisterId = Principal.anonymous();
  let resume, observed;
  const paused = new Promise(resolve => { resume = resolve; });
  const agent = { async query(_target, fields) {
    await paused;
    observed = fields.arg;
    assert.equal(fields.effectiveCanisterId, effectiveCanisterId);
    return { status: 'replied', reply: { arg: new Uint8Array() } };
  } };
  const pending = queryRaw(agent, Principal.managementCanister(), {
    methodName: 'raw_method', arg: backing.subarray(1, 4), effectiveCanisterId,
  });
  backing.fill(0);
  resume();
  await pending;
  assert.deepEqual([...observed], [5, 6, 7]);
});

test('query replies own selected Buffer bytes after returning', async () => {
  const backing = Buffer.from([99, 8, 7, 6, 99]);
  const agent = { async query() {
    return { status: 'replied', reply: { arg: backing.subarray(1, 4) } };
  } };
  const reply = await queryRaw(agent, Principal.anonymous(), { methodName: 'raw_method', arg: new Uint8Array() });
  backing.fill(0);
  assert.deepEqual([...reply], [8, 7, 6]);
  assert(!Buffer.isBuffer(reply));
});

test('raw query byte-shape failures do not coerce values or dispatch invalid arguments', async () => {
  let calls = 0;
  const agent = { async query() {
    calls++;
    return { status: 'replied', reply: { arg: new ArrayBuffer(3) } };
  } };
  for (const arg of [undefined, null, 'payload', [1, 2], new ArrayBuffer(3), new DataView(new ArrayBuffer(3))]) {
    await assert.rejects(queryRaw(agent, Principal.anonymous(), { methodName: 'raw_method', arg }), TypeError);
  }
  assert.equal(calls, 0);
  await assert.rejects(queryRaw(agent, Principal.anonymous(), { methodName: 'raw_method', arg: new Uint8Array() }), TypeError);
  assert.equal(calls, 1);
});

test('raw queries preserve opaque payload bytes and SDK signing/request identity', async () => {
  const argument = Uint8Array.of(0x49, 0x43, 0x57, 0, 7);
  const reply = Uint8Array.of(9, 8, 7);
  const target = Principal.anonymous();
  let requests = 0, content;
  const agent = await HttpAgent.create(createAgentOptions(options({ fetch: async (url, init) => {
    requests++;
    assert.equal(new URL(url).pathname, `/api/v3/canister/${target.toText()}/query`);
    const envelope = Cbor.decode(new Uint8Array(init.body));
    content = envelope.content;
    assert.deepEqual(new Uint8Array(content.arg), argument);
    assert.equal(content.method_name, 'wire_method');
    assert(envelope.sender_sig.length > 0);
    return new Response(Cbor.encode({ status: 'replied', reply: { arg: reply } }),
      { headers: { 'content-type': 'application/cbor' } });
  } })));
  assert.deepEqual(await queryRaw(agent, target, { methodName: 'wire_method', arg: argument }), reply);
  assert.equal(requests, 1);
  assert.equal(requestIdOf(content).length, 32);
});

test('query failures retain upstream types and context without retrying', async () => {
  const target = Principal.anonymous();
  const requestId = new Uint8Array(32).fill(9);
  const httpDetails = { ok: true, status: 200, statusText: 'OK', headers: [] };
  let calls = 0;
  const agent = { async query() {
    calls++;
    return { status: 'rejected', requestId, reject_code: 4, reject_message: 'denied',
      error_code: 'IC0406', signatures: [], httpDetails };
  } };
  for (const fields of [{}, { effectiveCanisterId: 'aaaaa-aa' },
    { effectiveTarget: { canisterId: 'aaaaa-aa' } },
    { effectiveTarget: { subnetId: 'aaaaa-aa' } }]) {
    await assert.rejects(queryRaw(agent, target, { ...fields, methodName: 'wire_method', arg: new Uint8Array() }), error => {
      assert(error instanceof RejectError);
      assert.equal(error.code.callContext.canisterId, target);
      assert.equal(error.code.callContext.methodName, 'wire_method');
      assert.equal(error.code.callContext.httpDetails, httpDetails);
      const route = fields.effectiveTarget?.subnetId ? 'subnetId' : 'canisterId';
      assert.equal(error.code.callContext.effectiveTarget[route].toText(),
        Object.keys(fields).length ? 'aaaaa-aa' : target.toText());
      return true;
    });
  }
  assert.equal(calls, 4);
  const failure = new Error('transport');
  await assert.rejects(queryRaw({ query: async () => { throw failure; } }, target,
    { methodName: 'wire_method', arg: new Uint8Array() }), error => error === failure);
});

test('external Candid bytes remain the external SDK contract', () => {
  const bytes = IDL.encode([IDL.Text], ['root']);
  assert.equal(Buffer.from(bytes).toString('hex'), '4449444c00017104726f6f74');
  assert.deepEqual(IDL.decode([IDL.Text], bytes), ['root']);
});
