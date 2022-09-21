import { Client } from 'rpc-websockets'
export { Client } from 'rpc-websockets'

export type ClientId = string
export type ClientOptions = ConstructorParameters<typeof Client>[1]

export async function connect(
  clientId: ClientId,
  origin?: string | null,
  token?: string | null,
  clientOptions: ClientOptions = {}
): Promise<Client> {
  const baseUrl =
    typeof origin === 'string'
      ? origin
      : `${window.location.protocol.replace('http', 'ws')}//${
          window.location.host
        }`

  let connectUrl = `${baseUrl}/~rpc?client=${clientId}`
  if (typeof token === 'string' && token.length > 0) {
    connectUrl += `&token=${token}`
  }

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
