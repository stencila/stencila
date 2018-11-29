import {Readable, Writable, PassThrough} from 'stream'

import StdioClient from '../../src/comms/StdioClient'
import Processor from '../../src/Processor'
import Person from '../../src/types/Person'
import StdioServer from '../../src/comms/StdioServer';

test('Stdio', async () => {
  const serverStdin = new PassThrough()
  const serverStdout = new PassThrough()

  const server = new StdioServer(undefined, undefined, serverStdin, serverStdout)
  server.start()

  const client = new StdioClient(serverStdout, serverStdin)

  const processor = new Processor()

  expect(await client.manifest()).toEqual(processor.manifest())

  const string = '{"type": "Person", "givenNames": ["John", "Paul"], "familyNames": ["Smith"]}'
  const object = JSON.parse(string)
  const thing = new Person(object)
    
  let args
  args = [string, 'application/ld+json']
  //@ts-ignore
  expect(await client.execute(...args)).toEqual(processor.execute(...args))

  for (let method of ['import', 'compile', 'build', 'execute']) {
    //@ts-ignore
    expect(await client[method](string)).toEqual(thing)
    //@ts-ignore
    expect(await client[method](object)).toEqual(thing)
    //@ts-ignore
    expect(await client[method](thing)).toEqual(thing)
    
    try {
      //@ts-ignore
      await client[method]('foo', 'bar/baz')
    } catch (error) {
      expect(error.message).toEqual("Internal error: Unhandled import format: bar/baz")
    }
  }

  // There should be no more requests waiting for a response
  // @ts-ignore
  expect(Object.keys(client.requests).length).toEqual(0)

  server.stop()
})
