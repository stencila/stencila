import { Client } from 'rpc-websockets'
import { ProjectId } from './types'
export { Client } from 'rpc-websockets'

export type ClientId = string
export type ClientOptions = ConstructorParameters<typeof Client>[1]

export async function connect(
  projectId: ProjectId,
  clientId: ClientId,
  origin?: string | null,
  clientOptions: ClientOptions = {}
): Promise<Client> {
  const baseUrl =
    typeof origin === 'string'
      ? origin
      : `${window.location.protocol.replace('http', 'ws')}//${
          window.location.host
        }`
  const connectUrl = `${baseUrl}/${projectId}?client=${clientId}`

  const client = new Client(connectUrl, {
    reconnect_interval: 1000 + 3000 * Math.random(), // random interval between 1 and 4 seconds
    max_reconnects: 300, // attempt to reconnect for ~5-15 minutes
    ...clientOptions,
  })
  return new Promise<Client>((resolve) =>
    client.on('open', () => resolve(client))
  )
}

export function disconnect(client: Client): void {
  client.close()
}
