import StdioServer from '../../src/comms/StdioServer'

test('run', () => {
  const server = new StdioServer()
  server.run()
  
  expect(server.stdio).toBeDefined()
  
  server.stop()
  
  expect(server.stdio).toBeUndefined()
})

test('log:none', () => {
  const stdout = jest.spyOn(process.stdout, 'write')
  const stderr = jest.spyOn(process.stderr, 'write')
  
  const server = new StdioServer()
  server.start()
  
  server.stdio!.emit('line', 'foo')
  server.stdio!.emit('line', '{}')
  server.stdio!.emit('line', '{"id":1, "method":"import", "params":[{"type":"Thing","name":"Joe"}]}')
  
  server.stdio!.on('close', () => {
    expect(stdout.mock.calls.length).toEqual(3)
    expect(stdout.mock.calls).toEqual([
      ['{"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":"Parse error: Unexpected token o in JSON at position 1"}}\n'],
      ['{"jsonrpc":"2.0","id":null,"error":{"code":-32600,"message":"Invalid request: missing \\"method\\" property"}}\n'],
      ['{"jsonrpc":"2.0","id":1,"result":{"type":"Thing","name":"Joe"}}\n']
    ])
    expect(stderr.mock.calls).toEqual([])
  })

  server.stop()
})

test('log:all', () => {
  const stderr = jest.spyOn(process.stderr, 'write')

  const server = new StdioServer(undefined, 0)
  server.start()

  server.stdio!.emit('line', 'parse error')
  server.stdio!.emit('line', '{"invalid request": true}')
  server.stdio!.emit('line', '{"id":1, "method":"import", "params":[{"type":"Person","name":"Kate"}]}')
  
  server.stdio!.on('close', () => {
    expect(stderr.mock.calls.length).toEqual(3)
    
    const first = JSON.parse(stderr.mock.calls[0][0])
    expect(first.timestamp).toBeTruthy()
    expect(first.request).toBeTruthy()
    expect(first.response).toBeTruthy()
    
    const second = JSON.parse(stderr.mock.calls[1][0])
    expect(second.response.error).toEqual({"code": -32600, "message": "Invalid request: missing \"method\" property"})
    expect(second.response.result).toBeUndefined()
    
    const third = JSON.parse(stderr.mock.calls[2][0])
    expect(third.response.error).toBeUndefined()
    expect(third.response.result).toEqual({"type": "Person", "name": "Kate"})
  })

  server.stop()
})
