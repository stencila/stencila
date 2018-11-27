// This does some pretty funky things with types so use JS instead of TS

const childProcess = require('child_process')
const path = require('path')

const StdioClient = require('../../src/comms/StdioClient').default
const Processor = require('../../src/Processor').default
const Person = require('../../src/types/Person').default

// Setup and teardown to ensure server is terminated even if test fails

let server

beforeEach(() => {
  server = childProcess.spawn('npx', ['ts-node', 'tests/comms/stdioServer.ts'], {
    cwd: path.join(__dirname, '..', '..')
  })
})

afterEach(() => {
  server.kill('SIGTERM')
})

test('StdioClient', async () => {
  const processor = new Processor()
  const client = new StdioClient(server.stdin, server.stdout)

  expect(await client.manifest()).toEqual(processor.manifest())

  const string = '{"type": "Person", "givenNames": ["John", "Paul"], "familyNames": ["Smith"]}'
  const object = JSON.parse(string)
  const thing = new Person(object)
    
  let args
  
  args = [string, 'application/ld+json']
  expect(await client.import(...args)).toEqual(processor.import(...args))

  for (let method of ['import', 'compile', 'build', 'execute']) {
    expect(await client[method](string)).toEqual(thing)
    expect(await client[method](object)).toEqual(thing)
    expect(await client[method](thing)).toEqual(thing)
    
    try {
      await client[method]('foo', 'bar/baz')
    } catch (error) {
      expect(error.message).toEqual("Internal error: Unhandled import format: bar/baz")
    }
  }

  // There should be no more requests waiting for a response
  expect(Object.keys(client.requests).length).toEqual(0)
})
