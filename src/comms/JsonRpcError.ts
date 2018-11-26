/**
 * A JSON-RPC 2.0 response error
 *
 * @see {@link https://www.jsonrpc.org/specification#error_object}
 */
export default class JsonRpcError {
  /**
   * A Number that indicates the error type that occurred.
   * This MUST be an integer.
   */
  code: number

  /**
   * A String providing a short description of the error.
   * The message SHOULD be limited to a concise single sentence.
   */
  message: string

  /**
   * A Primitive or Structured value that contains additional information about the error.
   * This may be omitted.
   * The value of this member is defined by the Server (e.g. detailed error information,
   * nested errors etc.).
   */
  data?: any

  constructor (code: number, message: string, data?: any) {
    this.code = code
    this.message = message
    this.data = data
  }
}
