import HttpClient from '../../../src/comms/HttpClient'
import HttpServer from '../../../src/comms/HttpServer'
import Person from '../../../src/types/Person'

test('HTTP', async () => {
  const server = new HttpServer()
  await server.start()

  const client = new HttpClient(`http://${server.address}:${server.port}`)
  
  const string = '{"type": "Person", "givenNames": ["John", "Paul"], "familyNames": ["Smith"]}'
  const object = JSON.parse(string)
  const thing = new Person(object)
  
  // JSON-RPC over HTPP

  expect(await client.execute(string)).toEqual(thing)
  expect(await client.execute(object)).toEqual(thing)
  expect(await client.execute(thing)).toEqual(thing)
  
  try {
    await client.execute('foo', 'bar/baz')
  } catch (error) {
    expect(error.message).toEqual("Internal error: Unhandled import format: bar/baz")
  }

  // JSON-RPC wrapped in HTTP

  // This is currently not working! The whole `Thing` serialisation/deserialisation
  // flow needs refactoring
  //expect(await client.post('execute', {thing: string})).toEqual(thing)
  //expect(await client.post('execute', {thing: object})).toEqual(thing)
  //expect(await client.post('execute', {thing: thing})).toEqual(thing)

  try {
    await client.post('execute', {thing: string, format: 'bar/baz'})
  } catch (error) {
    expect(error.message).toEqual("Internal error: Unhandled import format: bar/baz")
  }

  await server.stop()
})
