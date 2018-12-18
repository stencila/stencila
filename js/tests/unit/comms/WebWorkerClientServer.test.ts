import WebWorkerClient from '../../../src/comms/WebWorkerClient'
import WebWorkerServer from '../../../src/comms/WebWorkerServer'
import Person from '../../../src/types/Person'

test('WebWorkers', async () => {

  // Mock workers to enable testing in Node.js
  // This simulates posting messages between the main
  // and worker threads.

  class ClientWorker {
    postMessage(msg: any) {
      // @ts-ignore
      self.onmessage({data: msg})
    }
  }
  const clientWorker = new ClientWorker()

  class ServerWorker {
    postMessage(msg: any) {
      // @ts-ignore
      clientWorker.onmessage({data: msg})
    }
  }
  // In the WebWorker thread, the global `self` is
  // used for `postMessage` and `onmessage`
  // @ts-ignore
  global.self = new ServerWorker()

  // End of mocks. Let the tests begin!

  const server = new WebWorkerServer()
  await server.start()

  //@ts-ignore
  const client = new WebWorkerClient(clientWorker)

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

  await server.stop()
})
