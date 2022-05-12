import { Client, connect, disconnect } from './client'
import { languages } from './kernels'

jest.setTimeout(10000)

const clientId = 'cl-kernels-tests'
let client: Client
beforeAll(async () => {
  client = await connect(
    clientId,
    process.env.SERVER_URL ?? 'ws://127.0.0.1:9000'
  )
})
afterAll(() => {
  disconnect(client)
})

test('basic', async () => {
  const kernels = await languages(client, 'sessionId')
  expect(kernels).toEqual(expect.arrayContaining(['Calc']))
})
