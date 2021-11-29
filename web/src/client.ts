import { Client } from 'rpc-websockets'
import { ProjectId } from './types'
export { Client } from 'rpc-websockets'

export type ClientId = string

export async function connect(
  projectId: ProjectId,
  clientId: ClientId,
  origin?: string | null,
  token?: string | null
): Promise<Client> {
  const baseUrl =
    typeof origin === 'string'
      ? origin
      : `${window.location.protocol.replace('http', 'ws')}//${
          window.location.host
        }`
  let connectUrl = `${baseUrl}/${projectId}?client=${clientId}`
  if (typeof token === 'string' && token.length > 0)
    connectUrl += `&token=${token}`

  const client = new Client(connectUrl)
  return new Promise<Client>((resolve) =>
    client.on('open', () => resolve(client))
  )
}

export function disconnect(client: Client): void {
  client.close()
}
