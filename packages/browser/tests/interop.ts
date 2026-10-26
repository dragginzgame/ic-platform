import type { HttpAgent, Identity } from '@dragginzgame/ic-platform/agent';
import type { StorageClient } from '@caffeineai/object-storage';
import type { AuthClient } from '@icp-sdk/auth/client';

// Compile the actual provider boundaries without making storage/login calls.
export function caffeineAgent(agent: HttpAgent): ConstructorParameters<typeof StorageClient>[4] {
  return agent;
}
export function authIdentity(auth: AuthClient): Promise<Identity> {
  return auth.getIdentity();
}
