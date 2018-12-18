import JsonRpcRequest from './JsonRpcRequest'
import JsonRpcResponse from './JsonRpcResponse'

/**
 * Base message encoder class
 */
export default abstract class Encoder {

  /**
   * Get the name of the encoder
   */
  abstract name (): string

  /**
   * Encode a JSON-RPC message
   *
   * @param instance The request or response message to encode
   * @returns A string or binary encoding of the message
   */
  abstract encode (instance: JsonRpcRequest | JsonRpcResponse): string | Uint8Array

  /**
   * Decode a JSON-RPC message
   *
   * @param message The request or response message to decode
   * @param constructor `JsonRpcRequest` or `JsonRpcResponse` class
   * @returns A new `JsonRpcRequest` or `JsonRpcResponse` instance
   */
  abstract decode<Type = JsonRpcRequest | JsonRpcResponse> (message: string | Uint8Array, constructor: new () => Type): Type

}
