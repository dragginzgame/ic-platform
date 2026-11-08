import assert from 'node:assert/strict';
import test from 'node:test';
import { readFileSync } from 'node:fs';
import { Cbor, Endpoint, Expiry, requestIdOf } from '@icp-sdk/core/agent';
import { Ed25519KeyIdentity } from '@icp-sdk/core/identity';
import { Principal } from '@icp-sdk/core/principal';
import { inspectCallIngress, inspectIngress } from '../dist/agent.js';
import { callWithMethodFixture, corruptCertificateSignatureFixture, readStateResponseFixture, invalidCertificateFixture, queryReplyFixture } from '../dist/testing.js';

test('shared Rust ingress corpus preserves full-content request identity', () => {
  const corpus = readFileSync(new URL('../../../crates/ic-platform/tests/fixtures/ingress.tsv', import.meta.url), 'utf8');
  for (const line of corpus.trim().split('\n').filter(line => !line.startsWith('#'))) {
    const [name, hex, , accepted, expiry, id] = line.split('\t');
    const bytes = new Uint8Array(Buffer.from(hex, 'hex'));
    const before = bytes.slice();
    if (accepted === 'true') {
      const result = inspectCallIngress(bytes);
      assert.equal(String(result.ingressExpiry), expiry, name);
      assert.equal(Buffer.from(result.requestId).toString('hex'), id, name);
    } else {
      assert.throws(() => inspectCallIngress(bytes), TypeError, name);
    }
    assert.deepEqual(bytes, before, name);
  }
});

function content(overrides = {}) {
  return {
    request_type: 'call', sender: Principal.anonymous().toUint8Array(),
    canister_id: new Uint8Array(), method_name: '_immutableObjectStorageCreateCertificate',
    arg: Uint8Array.of(0x44, 0x49, 0x44, 0x4c), ingress_expiry: 18446744073709551615n,
    ...overrides,
  };
}

test('inspects SDK-signed call bytes with exact Principal, payload and request identity', async () => {
  const identity = Ed25519KeyIdentity.fromSecretKey(new Uint8Array(32).fill(7));
  const body = content({ sender: identity.getPrincipal(), canister_id: Principal.anonymous(),
    ingress_expiry: Expiry.fromJSON('{"__expiry__":"18446744073709551615"}'), nonce: Uint8Array.of(8, 9) });
  const signed = await identity.transformRequest({ endpoint: Endpoint.Call, request: { method: 'POST' }, body });
  const envelope = Cbor.encode(signed.body);
  const before = envelope.slice();
  const inspected = inspectCallIngress(envelope);
  assert(inspected.sender instanceof Principal);
  assert.equal(inspected.sender.toText(), identity.getPrincipal().toText());
  assert.equal(inspected.canisterId.toText(), '2vxsx-fae');
  assert.equal(inspected.methodName, body.method_name);
  assert.deepEqual(inspected.arg, body.arg);
  assert.equal(inspected.ingressExpiry, 18446744073709551615n);
  assert.deepEqual(inspected.nonce, body.nonce);
  assert.deepEqual(inspected.requestId, requestIdOf(body));
  assert.deepEqual(envelope, before);
  // Inspection does not claim authentication or change delegation/signature bytes.
  const unverified = { ...signed.body, sender_sig: Uint8Array.of(0),
    sender_delegation: [{ delegation: { pubkey: Uint8Array.of(1), expiration: 1n }, signature: Uint8Array.of(2) }] };
  assert.deepEqual(inspectCallIngress(Cbor.encode(unverified)).requestId, inspected.requestId);
});

test('anonymous calls retain integer precision, full-content hashing and selected byte snapshots', () => {
  for (const expiry of [0, Number.MAX_SAFE_INTEGER, 9007199254740993n, 18446744073709551615n]) {
    const body = content({ ingress_expiry: expiry, nonce: Uint8Array.of(8), extension: Uint8Array.of(9) });
    const encoded = Cbor.encode({ content: body });
    const backing = Buffer.concat([Buffer.of(99), Buffer.from(encoded), Buffer.of(99)]);
    const inspected = inspectCallIngress(backing.subarray(1, -1));
    backing.fill(0);
    assert.equal(inspected.sender.toText(), '2vxsx-fae');
    assert.equal(inspected.canisterId.toText(), 'aaaaa-aa');
    assert.equal(inspected.ingressExpiry, BigInt(expiry));
    assert.deepEqual(inspected.arg, body.arg);
    assert.deepEqual(inspected.nonce, body.nonce);
    assert.deepEqual(inspected.requestId, requestIdOf(body));
    assert.notDeepEqual(inspected.requestId, requestIdOf(content({ ingress_expiry: expiry })));
    for (const bytes of [inspected.arg, inspected.nonce, inspected.requestId]) assert(!Buffer.isBuffer(bytes));
    inspected.arg.fill(0);
    assert.deepEqual(inspectCallIngress(encoded).arg, body.arg);
  }
  const noNonce = inspectCallIngress(Cbor.encode({ content: content() }));
  assert.equal(Object.hasOwn(noNonce, 'nonce'), false);
});

test('malformed or non-call ingress is refused without leaking request bytes or SDK causes', () => {
  const invalidContent = [null, [], 'private-payload',
    ...Object.keys(content()).map(key => { const body = content(); delete body[key]; return body; }),
    ...[ 'query', 'read_state', null ].map(request_type => content({ request_type })),
    ...[ [], '2vxsx-fae', new Uint8Array(30) ].flatMap(bytes =>
      [content({ sender: bytes }), content({ canister_id: bytes })]),
    content({ method_name: 1 }), content({ arg: [1, 2] }), content({ arg: 'private-payload' }),
    content({ nonce: null }), content({ nonce: [8] }),
    ...[-1, -1n, '18446744073709551615', null].map(ingress_expiry => content({ ingress_expiry })),
  ];
  // Force a floating-point wire representation; the SDK encoder writes large
  // integers losslessly, so encoding an unsafe JS integer is not this fixture.
  const zeroExpiry = Cbor.encode({ content: content({ ingress_expiry: 0 }) });
  const floatEnvelopes = [1.5, Number.MAX_SAFE_INTEGER + 1, Infinity, NaN].map(value => {
    const floatExpiry = new Uint8Array(9);
    floatExpiry[0] = 0xfb;
    new DataView(floatExpiry.buffer).setFloat64(1, value);
    return Uint8Array.from([...zeroExpiry.subarray(0, -1), ...floatExpiry]);
  });
  const inputs = [undefined, null, 'private-payload', [], new ArrayBuffer(0), new Uint8Array(),
    Uint8Array.of(255), ...[null, [], {}, { content: [] }].map(value => Cbor.encode(value)),
    ...invalidContent.map(body => Cbor.encode({ content: body })),
    ...floatEnvelopes];
  for (const [index, input] of inputs.entries()) {
    assert.throws(() => inspectCallIngress(input), error => {
      assert(error instanceof TypeError);
      assert.match(error.message, /^Expected an IC (call )?ingress envelope$/);
      assert.equal(error.cause, undefined);
      return true;
    }, `malformed fixture ${index}`);
  }
});

test('query and read-state inspection preserves signing metadata and exact paths', async () => {
  const identity = Ed25519KeyIdentity.fromSecretKey(new Uint8Array(32).fill(7));
  for (const body of [content({ request_type: 'query', sender: identity.getPrincipal() }), {
    request_type: 'read_state', sender: identity.getPrincipal(), ingress_expiry: 10n,
    paths: [[Uint8Array.of(0, 255), Uint8Array.of(1)]],
  }]) {
    const signed = await identity.transformRequest({ endpoint: body.request_type === 'query' ? Endpoint.Query : Endpoint.ReadState,
      request: { method: 'POST' }, body });
    const bytes = Cbor.encode(signed.body);
    const inspected = inspectIngress(bytes);
    assert.equal(inspected.requestType, body.request_type);
    assert.deepEqual(inspected.senderPublicKey, new Uint8Array(identity.getPublicKey().toDer()));
    assert.deepEqual(inspected.senderSignature, new Uint8Array(signed.body.sender_sig));
    assert.deepEqual(inspected.requestId, requestIdOf(body));
    if (inspected.requestType === 'read_state') assert.deepEqual(inspected.paths, body.paths);
    assert.throws(() => inspectCallIngress(bytes), TypeError);
    bytes.fill(0);
    assert.deepEqual(inspected.senderPublicKey, new Uint8Array(identity.getPublicKey().toDer()));
  }
});

test('typed negative fixtures retain the fields that consumer refusal tests protect', () => {
  const call = Cbor.encode({ content: content(), sender_sig: Uint8Array.of(9) });
  const changed = Cbor.decode(callWithMethodFixture(call, 'unrelated'));
  assert.equal(changed.content.method_name, 'unrelated');
  assert.deepEqual(changed.content.arg, content().arg);
  assert.deepEqual(changed.sender_sig, Uint8Array.of(9));
  const cert = { tree: [0], signature: Uint8Array.of(7, 8), delegation: { certificate: Uint8Array.of(3) } };
  const corrupted = Cbor.decode(corruptCertificateSignatureFixture(Cbor.encode(cert)));
  assert.deepEqual(corrupted, { ...cert, signature: Uint8Array.of(6, 8) });
  assert.deepEqual(Cbor.decode(invalidCertificateFixture()), {});
});

test('shared Rust response corpus preserves both framing choices and opaque bytes', () => {
  const corpus = readFileSync(new URL('../../../crates/ic-platform/tests/fixtures/protocol.tsv', import.meta.url), 'utf8');
  const seen = new Set();
  for (const row of corpus.trim().split('\n').filter(row => !row.startsWith('#'))) {
    assert.equal(row.split('\t').length, 4);
    const [kind, producer, hex, payloadHex] = row.split('\t');
    const key = `${kind}/${producer}`;
    assert(!seen.has(key), `duplicate corpus case: ${key}`);
    seen.add(key);
    for (const text of [hex, payloadHex]) assert.match(text, /^(?:[0-9a-f]{2})*$/);
    const bytes = Uint8Array.from(Buffer.from(hex, 'hex'));
    const payload = Uint8Array.from(Buffer.from(payloadHex, 'hex'));
    const before = bytes.slice();
    const decoded = Cbor.decode(bytes);
    let encoded;
    if (kind === 'read-state') {
      assert.deepEqual(decoded, { certificate: payload });
      encoded = readStateResponseFixture(payload);
    } else {
      assert.equal(kind, 'query-reply');
      assert.deepEqual(decoded, { status: 'replied', reply: { arg: payload } });
      encoded = queryReplyFixture(payload);
    }
    if (producer === 'sdk') assert.deepEqual(encoded, bytes, row);
    else { assert.equal(producer, 'rust'); assert.deepEqual(encoded.subarray(3), bytes, row); }
    assert.deepEqual(bytes, before);
  }
  assert.deepEqual([...seen].sort(), ['query-reply/rust', 'query-reply/sdk', 'read-state/rust', 'read-state/sdk']);
});
