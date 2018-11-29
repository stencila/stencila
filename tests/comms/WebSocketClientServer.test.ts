import WebSocketClient from '../../src/comms/WebSocketClient'
import WebSocketServer from '../../src/comms/WebSocketServer'
import Person from '../../src/types/Person'

test('WebSockets', async () => {
  const server = new WebSocketServer()
  await server.start()

  const client = new WebSocketClient(`ws://${server.address}:${server.port}`)
  await new Promise(resolve => setTimeout(resolve, 1000))
  
  const string = '{"type": "Person", "givenNames": ["John", "Paul"], "familyNames": ["Smith"]}'
  const object = JSON.parse(string)
  const thing = new Person(object)
  
  expect(await client.import(string)).toEqual(thing)
  expect(await client.import(object)).toEqual(thing)
  expect(await client.import(thing)).toEqual(thing)
  
  try {
    await client.import('foo', 'bar/baz')
  } catch (error) {
    expect(error.message).toEqual("Internal error: Unhandled import format: bar/baz")
  }

  await server.stop()
})
