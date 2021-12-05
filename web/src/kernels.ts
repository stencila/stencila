import { Client } from './client'
import { SessionId } from './sessions'

/**
 * Get a list of language kernels available in the current environment
 */
export async function languages(
  client: Client,
  sessionId: SessionId
): Promise<string[]> {
  return client.call('kernels.languages', { sessionId }) as Promise<string[]>
}
