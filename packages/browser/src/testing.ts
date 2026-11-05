/** IC protocol fixtures only. Never infer authentication or dispatch authority. */
import { Cbor } from '@icp-sdk/core/agent';
import { inspectCallIngress } from './agent.js';

/** Unsigned query reply with exact opaque argument bytes. */
export function queryReplyFixture(arg: Uint8Array): Uint8Array {
  if (!(arg instanceof Uint8Array)) throw new TypeError('Expected reply bytes');
  return Cbor.encode({ status: 'replied', reply: { arg: new Uint8Array(arg) } });
}

/** Read-state response with caller-selected certificate bytes, including invalid proofs. */
export function readStateResponseFixture(certificate: Uint8Array): Uint8Array {
  if (!(certificate instanceof Uint8Array)) throw new TypeError('Expected certificate bytes');
  return Cbor.encode({ certificate: new Uint8Array(certificate) });
}

/** Syntactically valid empty container, deliberately not an IC certificate. */
export function invalidCertificateFixture(): Uint8Array {
  return Cbor.encode({});
}

/** Corrupt one signature byte without changing the certificate tree/delegation. */
export function corruptCertificateSignatureFixture(certificate: Uint8Array): Uint8Array {
  const decoded = Cbor.decode<Record<string, unknown>>(new Uint8Array(certificate));
  if (!(decoded.signature instanceof Uint8Array) || decoded.signature.length === 0) {
    throw new TypeError('Expected certificate signature bytes');
  }
  decoded.signature[0] = decoded.signature[0]! ^ 1;
  return Cbor.encode(decoded);
}

/** Change a call method while retaining its now-invalid original signature. */
export function callWithMethodFixture(envelope: Uint8Array, methodName: string): Uint8Array {
  inspectCallIngress(envelope);
  if (typeof methodName !== 'string') throw new TypeError('Expected method name');
  const decoded = Cbor.decode<{ content: Record<string, unknown> }>(new Uint8Array(envelope));
  decoded.content.method_name = methodName;
  return Cbor.encode(decoded);
}
