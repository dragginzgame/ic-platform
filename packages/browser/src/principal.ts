import { Principal } from '@icp-sdk/core/principal';

// Preserve the SDK identity required by external browser clients.
export { Principal };

export class PrincipalTextError extends Error {
  constructor() {
    super('Expected canonical IC Principal text');
    this.name = 'PrincipalTextError';
  }
}

/** Bounded canonical representation only; roles and allowed ranges stay caller-owned. */
export function parseCanonicalPrincipal(value: unknown): Principal {
  if (typeof value !== 'string' || value.length > 63) throw new PrincipalTextError();
  try {
    const principal = Principal.fromText(value);
    if (principal.toText() === value) return principal;
  } catch {
    // Do not embed untrusted input in diagnostics.
  }
  throw new PrincipalTextError();
}
