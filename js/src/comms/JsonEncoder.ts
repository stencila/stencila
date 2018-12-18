import Encoder from './Encoder'
import JsonRpcRequest from './JsonRpcRequest'
import JsonRpcResponse from './JsonRpcResponse'

/**
 * A [JSON](http://json.org/) message encoder
 */
export default class JsonEncoder extends Encoder {

  name (): string {
    return 'json'
  }

  encode (instance: JsonRpcRequest | JsonRpcResponse): string {
    return JSON.stringify(instance)
  }

  decode<Type = JsonRpcRequest | JsonRpcResponse> (message: string, constructor: new () => Type): Type {
    const instance = new constructor()
    const data = JSON.parse(message)
    Object.assign(instance, data)
    return instance
  }
}
