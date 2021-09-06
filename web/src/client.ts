import { Client } from 'rpc-websockets'

export { Client } from 'rpc-websockets'

export async function connect(): Promise<Client> {
  let client = new Client(`ws://127.0.0.1:9000/~ws?client=clientId`)
  return new Promise<Client>((resolve) =>
    client.on('open', () => resolve(client))
  )
}

export function disconnect(client: Client) {
  client.close()
}
