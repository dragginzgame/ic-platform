import { Principal, parseCanonicalPrincipal } from '@tokostudios/ic-platform/principal';
import { Ed25519KeyIdentity, restoreEd25519Identity } from '@tokostudios/ic-platform/identity';
import { Secp256k1KeyIdentity, restoreSecp256k1Identity } from '@tokostudios/ic-platform/identity/secp256k1';
import { HttpAgent, createAgentOptions, queryRaw, inspectCallIngress, inspectIngress, IC_ROOT_KEY, type Identity, type CallIngressInspection } from '@tokostudios/ic-platform/agent';
import { Endpoint, SubmitRequestType, ReadRequestType, Expiry } from '@tokostudios/ic-platform/agent';
import { Certificate, lookupResultToBuffer } from '@tokostudios/ic-platform/certification';
import { queryReplyFixture } from '@tokostudios/ic-platform/testing';
import { IDL } from '@tokostudios/ic-platform/candid';

export async function smoke() {
  const key = Ed25519KeyIdentity.fromSecretKey(new Uint8Array(32).fill(7));
  const identity: Identity = restoreEd25519Identity(JSON.stringify(key.toJSON()), 512);
  const secp = Secp256k1KeyIdentity.fromSecretKey(new Uint8Array(32).fill(8));
  if (restoreSecp256k1Identity(JSON.stringify(secp.toJSON()), 512).getPrincipal().toText() !== secp.getPrincipal().toText()) {
    throw new Error('Secp identity mismatch');
  }
  const target: Principal = parseCanonicalPrincipal('2vxsx-fae');
  const expiry = Expiry.fromJSON('{"__expiry__":"100000000000"}');
  if (expiry.toBigInt() !== 100000000000n || !Expiry.isExpiry(expiry)) throw new Error('Expiry changed');
  const common = { sender: identity.getPrincipal(), ingress_expiry: expiry };
  const method = { canister_id: target, method_name: 'wire_method', arg: Uint8Array.of(1, 2) };
  const signingRequests: Parameters<Identity['transformRequest']>[0][] = [
    { endpoint: Endpoint.Call, request: { method: 'POST' },
      body: { ...common, ...method, request_type: SubmitRequestType.Call } },
    { endpoint: Endpoint.Query, request: { method: 'POST' },
      body: { ...common, ...method, request_type: ReadRequestType.Query } },
    { endpoint: Endpoint.ReadState, request: { method: 'POST' },
      body: { ...common, request_type: ReadRequestType.ReadState, paths: [[Uint8Array.of(1)]] } },
  ];
  for (const request of signingRequests) {
    const signed = await identity.transformRequest(request);
    if (typeof signed !== 'object' || signed === null || !('body' in signed) ||
        typeof signed.body !== 'object' || signed.body === null ||
        !('sender_sig' in signed.body) || !(signed.body.sender_sig instanceof Uint8Array) ||
        signed.body.sender_sig.length === 0) throw new Error('Signing request failed');
  }
  const rootKey = Uint8Array.from(IC_ROOT_KEY.match(/../g)!, pair => Number.parseInt(pair, 16));
  const arg = Uint8Array.of(0x49, 0x43, 0x57, 7);
  const reply = Uint8Array.of(3, 2, 1);
  let calls = 0;
  let inspected: CallIngressInspection | undefined;
  const agent = await HttpAgent.create(createAgentOptions({
    host: new URL('https://example.invalid'), rootKey, identity,
    retryTimes: 0, verifyQuerySignatures: false, shouldSyncTime: false,
    fetch: async (url, init) => {
      if (new URL(String(url)).pathname.endsWith('/call')) {
        inspected = inspectCallIngress(init!.body as Uint8Array);
        if (inspected.sender.toText() !== identity.getPrincipal().toText() ||
            inspected.canisterId.toText() !== target.toText() || inspected.methodName !== 'wire_method' ||
            String(inspected.arg) !== String(arg) || typeof inspected.ingressExpiry !== 'bigint') {
          throw new Error('Ingress binding changed');
        }
        return new Response(null, { status: 202 });
      }
      calls++;
      const query = inspectIngress(init!.body as Uint8Array);
      if (query.requestType !== 'query' || String(query.arg) !== String(arg) || !query.senderSignature?.length) {
        throw new Error('Query signing bytes changed');
      }
      return new Response(new Uint8Array(queryReplyFixture(reply)).buffer,
        { headers: { 'content-type': 'application/cbor' } });
    },
  }));
  const response = await queryRaw(agent, target, { methodName: 'wire_method', arg });
  if (String(response) !== String(reply) || calls !== 1) throw new Error('Unexpected dispatch');
  const submitted = await agent.call(target, { methodName: 'wire_method', arg, callSync: false });
  if (!inspected || String(inspected.requestId) !== String(submitted.requestId)) {
    throw new Error('Ingress request ID changed');
  }
  if (IDL.decode([IDL.Text], IDL.encode([IDL.Text], ['root']))[0] !== 'root') throw new Error('External Candid changed');
  let malformedRefused = false;
  try {
    await Certificate.create({ certificate: Uint8Array.of(255), rootKey, principal: { canisterId: target } });
  } catch { malformedRefused = true; }
  if (!malformedRefused || typeof lookupResultToBuffer !== 'function') throw new Error('Certificate boundary failed');
  return { principal: identity.getPrincipal().toText(), calls, malformedRefused };
}
