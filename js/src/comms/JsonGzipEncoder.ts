import pako from 'pako'

import Encoder from './Encoder'
import JsonEncoder from './JsonEncoder'
import JsonRpcRequest from './JsonRpcRequest'
import JsonRpcResponse from './JsonRpcResponse'

/**
 * A [JSON](http://json.org/) + gzip message encoder
 */
export default class JsonGzipEncoder extends Encoder {

  /**
   * Used to perform the underlying JSON encodong/decoding.
   * Unable to extend `JsonEncoder` because this handles binary,
   * rather than string, messages.
   */
  private json: JsonEncoder

  constructor () {
    super()
    this.json = new JsonEncoder()
  }

  name (): string {
    return 'json+gzip'
  }

  encode (instance: JsonRpcRequest | JsonRpcResponse): Uint8Array {
    const json = this.json.encode(instance)
    return pako.deflate(json)
  }

  decode<Type = JsonRpcRequest | JsonRpcResponse> (message: Uint8Array, constructor: new () => Type): Type {
    const json = pako.inflate(message, { to: 'string' })
    return this.json.decode(json, constructor)
  }
}
