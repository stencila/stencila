import pako from 'pako'

import CborEncoder from './CborEncoder'
import JsonRpcRequest from './JsonRpcRequest'
import JsonRpcResponse from './JsonRpcResponse'

/**
 * A [CBOR](http://cbor.io) + gzip message encoder
 */
export default class CborGzipEncoder extends CborEncoder {

  name (): string {
    return 'cbor+gzip'
  }

  encode (instance: JsonRpcRequest | JsonRpcResponse): Uint8Array {
    return pako.deflate(super.encode(instance))
  }

  decode<Type = JsonRpcRequest | JsonRpcResponse> (message: Uint8Array, constructor: new () => Type): Type {
    return super.decode(pako.inflate(message), constructor)
  }
}
