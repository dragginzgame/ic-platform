import {
  Cbor, requestIdOf, RejectError, UncertifiedRejectErrorCode,
  type Agent, type HttpAgentOptions, type Identity, type QueryFields,
} from '@icp-sdk/core/agent';
import { Principal } from '@icp-sdk/core/principal';

export { HttpAgent, IC_ROOT_KEY, requestIdOf, RejectError, UncertifiedRejectErrorCode } from '@icp-sdk/core/agent';
export { Endpoint, SubmitRequestType, ReadRequestType, Expiry } from '@icp-sdk/core/agent';
export type { Agent, Identity, HttpAgentOptions, SubmitResponse } from '@icp-sdk/core/agent';

interface IngressFields {
  sender: Principal;
  ingressExpiry: bigint;
  senderPublicKey?: Uint8Array;
  senderSignature?: Uint8Array;
  requestId: Uint8Array;
}

export interface CallIngressInspection extends IngressFields {
  requestType: 'call';
  canisterId: Principal;
  methodName: string;
  arg: Uint8Array;
  nonce?: Uint8Array;
}

export type IngressInspection = CallIngressInspection | (Omit<CallIngressInspection, 'requestType'> & {
  requestType: 'query';
}) | (IngressFields & { requestType: 'read_state'; paths: Uint8Array[][] });

/** Inspect IC ingress using the SDK decoder and request-ID hash, without IO.
 * Accepts signed or anonymous envelopes; does not authenticate signatures or
 * delegations, check freshness, or authorize dispatch. Callers retain input byte
 * limits and all request-binding/trust policies. Returned bytes are snapshots.
 * Malformed input produces a TypeError without ingress bytes or an SDK cause.
 */
export function inspectIngress(envelope: Uint8Array): IngressInspection {
  try {
    if (!(envelope instanceof Uint8Array)) throw new TypeError();
    const decoded = Cbor.decode<unknown>(new Uint8Array(envelope));
    if (!isRecord(decoded) || !isRecord(decoded.content)) throw new TypeError();
    const content = decoded.content;
    if (!(content.sender instanceof Uint8Array) || content.sender.length > 29) throw new TypeError();
    const expiry = content.ingress_expiry;
    if (!(typeof expiry === 'bigint' && expiry >= 0n) &&
        !(typeof expiry === 'number' && Number.isSafeInteger(expiry) && expiry >= 0)) throw new TypeError();
    const fields = {
      sender: Principal.fromUint8Array(new Uint8Array(content.sender)),
      ingressExpiry: BigInt(expiry),
      ...optionalBytes(decoded, 'sender_pubkey', 'senderPublicKey'),
      ...optionalBytes(decoded, 'sender_sig', 'senderSignature'),
      // Hash the complete decoded content, including nonce and extension fields.
      // Hashing a projection would silently change the signed request identity.
      requestId: requestIdOf(content),
    };
    if (content.request_type === 'read_state') {
      if (!Array.isArray(content.paths)) throw new TypeError();
      const paths = content.paths.map((path: unknown) => {
        if (!Array.isArray(path)) throw new TypeError();
        return path.map((label: unknown) => {
          if (!(label instanceof Uint8Array)) throw new TypeError();
          return new Uint8Array(label);
        });
      });
      return { ...fields, requestType: 'read_state', paths };
    }
    if ((content.request_type !== 'call' && content.request_type !== 'query') ||
        typeof content.method_name !== 'string' ||
        !(content.canister_id instanceof Uint8Array) || content.canister_id.length > 29 ||
        !(content.arg instanceof Uint8Array)) throw new TypeError();
    return { ...fields, requestType: content.request_type,
      canisterId: Principal.fromUint8Array(new Uint8Array(content.canister_id)),
      methodName: content.method_name, arg: new Uint8Array(content.arg),
      ...optionalBytes(content, 'nonce', 'nonce') };
  } catch {
    throw new TypeError('Expected an IC ingress envelope');
  }
}

/** Call-only inspection; retains the same structural-only trust boundary. */
export function inspectCallIngress(envelope: Uint8Array): CallIngressInspection {
  const inspected = inspectIngress(envelope);
  if (inspected.requestType !== 'call') throw new TypeError('Expected an IC call ingress envelope');
  return inspected;
}

function optionalBytes<K extends string>(record: Record<string, unknown>, key: string, field: K): Partial<Record<K, Uint8Array>> {
  const value = record[key];
  if (value === undefined) return {};
  if (!(value instanceof Uint8Array)) throw new TypeError();
  return { [field]: new Uint8Array(value) } as Record<K, Uint8Array>;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value) &&
    !(value instanceof Uint8Array);
}

export type ExplicitAgentOptions = Omit<HttpAgentOptions,
  'host' | 'rootKey' | 'identity' | 'retryTimes' | 'verifyQuerySignatures' |
  'shouldSyncTime' | 'shouldFetchRootKey'> & {
    host: URL;
    rootKey: Uint8Array;
    identity: Identity;
    retryTimes: number;
    verifyQuerySignatures: boolean;
    shouldSyncTime: boolean;
  };

/** Own trust bytes and require explicit policy before any SDK construction/IO.
 * URL scheme/host authorization, fetch limits and effect policy remain caller-owned.
 * Accepts SDK origin URLs; automatic root-key acquisition is always disabled.
 */
export function createAgentOptions(options: ExplicitAgentOptions): HttpAgentOptions {
  const { host, rootKey, ...rest } = options;
  if (!(host instanceof URL) || !['http:', 'https:'].includes(host.protocol) ||
      host.username || host.password || host.pathname !== '/' || host.search || host.hash) {
    throw new TypeError('Expected an admitted HTTP(S) origin URL');
  }
  if (!(rootKey instanceof Uint8Array) || rootKey.length === 0) throw new TypeError('Expected explicit trust-root bytes');
  if (typeof rest.identity?.getPrincipal !== 'function' || typeof rest.identity.transformRequest !== 'function') {
    throw new TypeError('Expected an explicit SDK-compatible identity');
  }
  if (!Number.isSafeInteger(rest.retryTimes) || rest.retryTimes < 0 ||
      typeof rest.verifyQuerySignatures !== 'boolean' || typeof rest.shouldSyncTime !== 'boolean') {
    throw new TypeError('Expected explicit retry, query verification and time-sync policies');
  }
  return { ...rest, host: host.origin, rootKey: new Uint8Array(rootKey), shouldFetchRootKey: false };
}

/** One SDK query invocation. No codec, additional retry or update/polling policy.
 * Rejections retain their upstream error identity and request/HTTP context.
 */
export async function queryRaw(
  agent: Pick<Agent, 'query'>,
  canisterId: Principal,
  fields: QueryFields,
): Promise<Uint8Array> {
  const { methodName } = fields;
  if (!(fields.arg instanceof Uint8Array)) throw new TypeError('Expected raw query argument bytes');
  const target = fields.effectiveTarget;
  const effectiveTarget = target
    ? 'canisterId' in target ? { canisterId: Principal.from(target.canisterId) }
      : { subnetId: Principal.from(target.subnetId) }
    : { canisterId: Principal.from(fields.effectiveCanisterId || canisterId) };
  const response = await agent.query(canisterId, { ...fields, arg: new Uint8Array(fields.arg) });
  if (response.status === 'replied') {
    if (!(response.reply.arg instanceof Uint8Array)) throw new TypeError('Expected raw query reply bytes');
    return new Uint8Array(response.reply.arg);
  }
  const code = new UncertifiedRejectErrorCode(response.requestId, response.reject_code,
    response.reject_message, response.error_code, response.signatures);
  code.callContext = { canisterId, methodName, effectiveTarget, httpDetails: response.httpDetails };
  throw RejectError.fromCode(code);
}
