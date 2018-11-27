import JsonRpcError from './JsonRpcError'

/**
 * A JSON-RPC 2.0 response
 *
 * @see {@link https://www.jsonrpc.org/specification#response_object}
 */
export default class JsonRpcResponse {
  /**
   * A string specifying the version of the JSON-RPC protocol. MUST be exactly "2.0".
   */
  jsonrpc: string = '2.0'

  /**
   * This member is REQUIRED.
   * It MUST be the same as the value of the id member in the Request Object.
   * If there was an error in detecting the id in the Request object (e.g. Parse error/Invalid Request), it MUST be Null.
   */
  id: number

  /**
   * This member is REQUIRED on success.
   * This member MUST NOT exist if there was an error invoking the method.
   * The value of this member is determined by the method invoked on the Server.
   */
  result?: any

  /**
   * This member is REQUIRED on error.
   * This member MUST NOT exist if there was no error triggered during invocation.
   * The value for this member MUST be an Object as defined in section 5.1.
   */
  error?: JsonRpcError

  constructor (id: number, result?: any, error?: JsonRpcError) {
    this.id = id
    this.result = result
    this.error = error
  }
}
