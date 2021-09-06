import { Client } from 'rpc-websockets'

export { Client } from 'rpc-websockets'

export async function connect(url: string): Promise<Client> {
  let client = new Client(`${url}?client=clientId`)
  return new Promise<Client>((resolve) =>
    client.on('open', () => resolve(client))
  )
}

export function disconnect(client: Client) {
  client.close()
}
