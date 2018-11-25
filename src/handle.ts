import { default as export_, exportObject } from './export'
import build from './build'
import compile from './compile'
import convert from './convert'
import execute from './execute'
import import_ from './import'
import manifest from './manifest'
import Thing from './Thing'

/**
 * A JSON-RPC 2.0 request
 *
 * @see {@link https://www.jsonrpc.org/specification#request_object}
 */
class Request {
  /**
   * A string specifying the version of the JSON-RPC protocol. MUST be exactly "2.0".
   */
  jsonrpc: string = '2.0'

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
   * An identifier established by the Client that MUST contain a string, number, or
   * NULL value if included. If it is not included it is assumed to be a notification.
   * The value SHOULD normally not be Null and numbers SHOULD NOT contain fractional
   * parts. The Server MUST reply with the same value in the Response object if included.
   * This member is used to correlate the context between the two objects.
   */
  id?: string | number | null
}

/**
 * A JSON-RPC 2.0 response
 *
 * @see {@link https://www.jsonrpc.org/specification#response_object}
 */
class Response {
  /**
   * A string specifying the version of the JSON-RPC protocol. MUST be exactly "2.0".
   */
  jsonrpc: string = '2.0'

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
  error?: ResponseError

  /**
   * This member is REQUIRED.
   * It MUST be the same as the value of the id member in the Request Object.
   * If there was an error in detecting the id in the Request object (e.g. Parse error/Invalid Request), it MUST be Null.
   */
  id: string | number | null = null

  constructor (result?: any, error?: ResponseError, id: string | number | null = null) {
    this.result = result
    this.error = error
    this.id = id
  }
}

/**
 * A JSON-RPC 2.0 response error
 *
 * @see {@link https://www.jsonrpc.org/specification#error_object}
 */
class ResponseError {
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

/**
 * Handle a JSON-RPC 2,0 request
 *
 * @see {@link Request}
 *
 * @param json A JSON-PRC request
 * @returns A JSON-RPC response
 */
export default function handle (json: string): string {
  let request: Request
  const response = new Response()

  // Extract a parameter by name from Object or by index from Array
  // tslint:disable-next-line:completed-docs
  function param (request: Request, index: number, name: string, required: boolean = true) {
    if (!request.params) throw new ResponseError(-32600, 'Invalid request: missing "params" property')
    const value = Array.isArray(request.params) ? request.params[index] : request.params[name]
    if (required && value === undefined) throw new ResponseError(-32602, `Invalid params: "${name}" is missing`)
    return value
  }

  try {
    // Parse JSON into an request
    try {
      request = JSON.parse(json)
    } catch (err) {
      throw new ResponseError(-32700, 'Parse error: ' + err.message)
    }

    // Response must always have an id
    response.id = request.id || null

    if (!request.method) throw new ResponseError(-32600, 'Invalid request: missing "method" property')

    let result
    switch (request.method) {
      case 'manifest':
        result = manifest()
        break
      case 'import':
        result = import_(
          param(request, 0, 'thing'),
          param(request, 1, 'format', false)
        )
        break
      case 'export':
        result = export_(
          param(request, 0, 'thing'),
          param(request, 1, 'format', false)
        )
        break
      case 'convert':
        result = convert(
          param(request, 0, 'thing'),
          param(request, 1, 'from', false),
          param(request, 2, 'to', false)
        )
        break
      case 'compile':
        result = compile(
          param(request, 0, 'thing'),
          param(request, 1, 'format', false)
        )
        break
      case 'build':
        result = build(
          param(request, 0, 'thing'),
          param(request, 1, 'format', false)
        )
        break
      case 'execute':
        result = execute(
          param(request, 0, 'thing'),
          param(request, 1, 'format', false)
        )
        break
      default:
        throw new ResponseError(-32601, `Method not found: "${request.method}"`)
    }

    // Most functions return a Thing tht needs to be exported to an Object
    // to include in the response JSON
    response.result = (result instanceof Thing) ? exportObject(result) : result
  } catch (exc) {
    response.error = (exc instanceof ResponseError) ? exc : new ResponseError(-32603, `Internal error: ${exc.message}`)
  }
  return JSON.stringify(response)
}
