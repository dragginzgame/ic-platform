import {
  RejectError, UncertifiedRejectErrorCode,
  type Agent, type HttpAgentOptions, type Identity, type QueryFields,
} from '@icp-sdk/core/agent';
import { Principal } from '@icp-sdk/core/principal';

export { HttpAgent, IC_ROOT_KEY, Cbor, requestIdOf, RejectError, UncertifiedRejectErrorCode } from '@icp-sdk/core/agent';
export type { Agent, Identity, HttpAgentOptions, SubmitResponse } from '@icp-sdk/core/agent';

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
