import { Client, connect, disconnect } from './client'
import { list } from './kernels'

jest.setTimeout(10000)

const clientId = 'cl-kernels-tests'
let client: Client
beforeAll(async () => {
  client = await connect(
    process.env.SERVER_URL ?? 'ws://127.0.0.1:9000/~ws',
    clientId
  )
})
afterAll(() => {
  disconnect(client)
})

test('basic', async () => {
  const kernels = await list(client, 'sessionId')
  expect(kernels).toEqual(expect.arrayContaining(['calc']))
})
