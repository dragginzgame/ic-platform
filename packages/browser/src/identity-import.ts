import type { Principal } from '@icp-sdk/core/principal';

export type IdentityImportCode = 'budget' | 'json' | 'key' | 'key-mismatch';

/** Deliberately excludes secret input and upstream exceptions containing that input. */
export class IdentityImportError extends Error {
  constructor(readonly code: IdentityImportCode) {
    super(`IC identity import refused: ${code}`);
    this.name = 'IdentityImportError';
  }
}

interface KeyIdentity {
  getPrincipal(): Principal;
  getKeyPair(): { secretKey: Uint8Array };
}

/** Shared import plumbing only. Key reconstruction and hashing remain SDK operations. */
export function importIdentity<T extends KeyIdentity>(
  sdk: { fromJSON(json: string): T; fromSecretKey(secret: Uint8Array): T },
  json: string,
  maxJsonBytes: number,
): T {
  if (!Number.isSafeInteger(maxJsonBytes) || maxJsonBytes <= 0) {
    throw new IdentityImportError('budget');
  }
  if (typeof json !== 'string' || json.length > maxJsonBytes ||
      new TextEncoder().encode(json).length > maxJsonBytes) throw new IdentityImportError('budget');
  let parsed: unknown;
  try { parsed = JSON.parse(json); } catch { throw new IdentityImportError('json'); }
  if (!Array.isArray(parsed) || parsed.length !== 2 ||
      !parsed.every(part => typeof part === 'string' && /^[0-9a-f]+$/.test(part)) ||
      parsed[1].length !== 64) throw new IdentityImportError('key');
  let advertised: T, reconstructed: T;
  try {
    advertised = sdk.fromJSON(json);
    reconstructed = sdk.fromSecretKey(advertised.getKeyPair().secretKey.slice());
  } catch { throw new IdentityImportError('key'); }
  if (reconstructed.getPrincipal().toText() !== advertised.getPrincipal().toText()) {
    throw new IdentityImportError('key-mismatch');
  }
  return reconstructed;
}
