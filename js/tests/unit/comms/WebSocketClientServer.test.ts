import WebSocketClient from '../../../src/comms/WebSocketClient'
import WebSocketServer from '../../../src/comms/WebSocketServer'
import Person from '../../../src/types/Person'

test('WebSockets', async () => {
  const server = new WebSocketServer()
  await server.start()

  const client = new WebSocketClient(`ws://${server.address}:${server.port}`)

  const string = '{"type": "Person", "givenNames": ["John", "Paul"], "familyNames": ["Smith"]}'
  const object = JSON.parse(string)
  const thing = new Person(object)
  
  expect(await client.execute(string)).toEqual(thing)
  expect(await client.execute(object)).toEqual(thing)
  expect(await client.execute(thing)).toEqual(thing)
  
  try {
    await client.execute('foo', 'bar/baz')
  } catch (error) {
    expect(error.message).toEqual("Internal error: Unhandled import format: bar/baz")
  }

  client.socket.close()
  await server.stop()
})
