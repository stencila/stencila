import { Client } from 'rpc-websockets'
export { Client } from 'rpc-websockets'

export type ClientId = string

export async function connect(
  url: string,
  clientId: ClientId
): Promise<Client> {
  let client = new Client(`${url}?client=${clientId}`)
  return new Promise<Client>((resolve) =>
    client.on('open', () => resolve(client))
  )
}

export function disconnect(client: Client) {
  client.close()
}
