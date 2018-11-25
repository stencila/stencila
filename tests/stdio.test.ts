test('stdio', () => {
  const spy = jest.spyOn(console, 'log')

  const {stdio} = require('../src/stdio')
  
  stdio.emit('line', 'foo')
  stdio.emit('line', '{}')
  stdio.emit('line', '{"id":1, "method":"import", "params":[{"type":"Thing","name":"Joe"}]}')
  
  stdio.on('close', () => {
    expect(spy.mock.calls).toEqual([
      ['{"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":"Parse error: Unexpected token o in JSON at position 1"}}'],
      ['{"jsonrpc":"2.0","id":null,"error":{"code":-32600,"message":"Invalid request: missing \\"method\\" property"}}'],
      ['{"jsonrpc":"2.0","id":1,"result":{"type":"Thing","name":"Joe"}}']
    ])
  })
  stdio.close()
})
