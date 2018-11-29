import HttpClient from '../../src/comms/HttpClient'
import HttpServer from '../../src/comms/HttpServer'
import Person from '../../src/types/Person'

test('HTTP', async () => {
  const server = new HttpServer()
  await server.start()

  const client = new HttpClient(`http://${server.address}:${server.port}`)
  
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
