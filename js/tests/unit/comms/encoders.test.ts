import 'jest'
import each from 'jest-each'

import JsonRpcRequest from '../../../src/comms/JsonRpcRequest'
import JsonRpcResponse from '../../../src/comms/JsonRpcResponse'

import CborEncoder from '../../../src/comms/CborEncoder'
import CborGzipEncoder from '../../../src/comms/CborGzipEncoder'
import JsonEncoder from '../../../src/comms/JsonEncoder'
import JsonGzipEncoder from '../../../src/comms/JsonGzipEncoder'

each([
  ['cbor', CborEncoder],
  ['cbor+gzip', CborGzipEncoder],
  ['json', JsonEncoder],
  ['json+gzip', JsonGzipEncoder]
]).describe('%p', (name: string, Encoder: any) => {
  const encoder = new Encoder()
  
  test('name', () => {
    expect(encoder.name()).toEqual(name)
  })

  test('roundtrip', () => {
    const request1 = new JsonRpcRequest('answer', ['life', 'universe', 'everything'])
    const request2 = encoder.decode(encoder.encode(request1), JsonRpcRequest)
    expect(request2).toEqual(request1)

    const response1 = new JsonRpcResponse(21, 42)
    const response2 = encoder.decode(encoder.encode(response1), JsonRpcResponse)
    expect(response2).toEqual(response1)
  })
})
