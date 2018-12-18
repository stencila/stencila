import cbor from 'cbor'

import Encoder from './Encoder'
import JsonRpcRequest from './JsonRpcRequest'
import JsonRpcResponse from './JsonRpcResponse'

/**
 * A [CBOR](http://cbor.io) message encoder
 */
export default class CborEncoder extends Encoder {

  name (): string {
    return 'cbor'
  }

  encode (instance: JsonRpcRequest | JsonRpcResponse): Uint8Array {
    return cbor.encode(instance)
  }

  decode<Type = JsonRpcRequest | JsonRpcResponse> (message: Uint8Array, constructor: new () => Type): Type {
    const instance = new constructor()
    const data = cbor.decodeFirstSync(Buffer.from(message))
    Object.assign(instance, data)
    return instance
  }
}
