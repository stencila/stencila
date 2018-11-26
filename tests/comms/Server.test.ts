import Server from '../../src/comms/Server'

// @ts-ignore Ignore that this is an abstract class
const server = new Server()

function check(request: null | object, response: object){
  expect(JSON.parse(server.handle(JSON.stringify(request)))).toEqual(response)
}

test('handle', () => {
  check(
    null, 
    {jsonrpc: '2.0', id: null, error: {code: -32603, message: "Internal error: Cannot read property 'id' of null"}}
  )

  check(
    {jsonrpc: '2.0'}, 
    {jsonrpc: '2.0', id: null, error: {code: -32600, message: 'Invalid request: missing "method" property'}}
  )

  check(
    {jsonrpc: '2.0', id: 1}, 
    {jsonrpc: '2.0', id: 1, error: {code: -32600, message: 'Invalid request: missing "method" property'}}
  )

  check(
    {jsonrpc: '2.0', id: 1, method: "foo"}, 
    {jsonrpc: '2.0', id: 1, error: {code: -32601, message: 'Method not found: "foo"'}}
  )

  check(
    {jsonrpc: '2.0', id: 1, method: "manifest"}, 
    {jsonrpc: '2.0', id: 1, result: server.processor.manifest()}
  )

  check(
    {jsonrpc: '2.0', id: 1, method: "import"}, 
    {jsonrpc: '2.0', id: 1, error: {code: -32600, message: 'Invalid request: missing "params" property'}}
  )

  check(
    {jsonrpc: '2.0', id: 1, method: "import", params: []}, 
    {jsonrpc: '2.0', id: 1, error: {code: -32602, message: 'Invalid params: "thing" is missing'}}
  )

  check(
    {jsonrpc: '2.0', id: 1, method: "import", params: ['{"type": "Thing"}']}, 
    {jsonrpc: '2.0', id: 1, result: {type: 'Thing'}}
  )

  check(
    {jsonrpc: '2.0', id: 1, method: "import", params: {thing: '{"type": "Thing"}'}}, 
    {jsonrpc: '2.0', id: 1, result: {type: 'Thing'}}
  )

  check(
    {jsonrpc: '2.0', id: 1, method: "export", params: {thing: '{"type": "Thing"}'}}, 
    {jsonrpc: '2.0', id: 1, result: '{"@context":"https://stencila.github.io/schema/context.jsonld","type":"Thing"}'}
  )

  check(
    {jsonrpc: '2.0', id: 1, method: "convert", params: ['{"type": "Thing"}']}, 
    {jsonrpc: '2.0', id: 1, result: '{"@context":"https://stencila.github.io/schema/context.jsonld","type":"Thing"}'}
  )

  check(
    {jsonrpc: '2.0', id: 1, method: "compile", params: ['{"type": "Thing"}']}, 
    {jsonrpc: '2.0', id: 1, result: {type: 'Thing'}}
  )

  check(
    {jsonrpc: '2.0', id: 1, method: "build", params: ['{"type": "Thing"}']}, 
    {jsonrpc: '2.0', id: 1, result: {type: 'Thing'}}
  )

  check(
    {jsonrpc: '2.0', id: 1, method: "execute", params: ['{"type": "Thing"}']}, 
    {jsonrpc: '2.0', id: 1, result: {type: 'Thing'}}
  )
})
