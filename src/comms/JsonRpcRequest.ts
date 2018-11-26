/**
 * A JSON-RPC 2.0 request
 *
 * @see {@link https://www.jsonrpc.org/specification#request_object}
 */
export default class JsonRpcRequest {
  /**
   * A string specifying the version of the JSON-RPC protocol. MUST be exactly "2.0".
   */
  jsonrpc: string = '2.0'

  /**
   * An identifier established by the Client that MUST contain a string, number, or
   * NULL value if included. If it is not included it is assumed to be a notification.
   * The value SHOULD normally not be Null and numbers SHOULD NOT contain fractional
   * parts. The Server MUST reply with the same value in the Response object if included.
   * This member is used to correlate the context between the two objects.
   */
  id: number

  /**
   * A string containing the name of the method to be invoked.
   * Method names that begin with the word rpc followed by a period character
   * (U+002E or ASCII 46) are reserved for rpc-internal methods and extensions and
   * MUST NOT be used for anything else.
   */
  method?: string

  /**
   * A structured value that holds the parameter values to be used during the
   * invocation of the method.This member MAY be omitted.
   */
  params?: {[key: string]: any} | any[]

  /**
   * A counter for generating unique, sequential request ids.
   *
   * Request ids don't need to be sequential but this helps with debugging.
   * Request ids don't need to be unique across clients.
   */
  static counter: number = 0

  constructor (method?: string, params?: any[]) {
    JsonRpcRequest.counter += 1
    this.id = JsonRpcRequest.counter
    this.method = method
    this.params = params
  }
}
