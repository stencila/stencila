import JsonRpcError from './JsonRpcError'
import JsonRpcRequest from './JsonRpcRequest'
import JsonRpcResponse from './JsonRpcResponse'
import Processor from '../Processor'
import Thing from '../types/Thing'

/**
 * A base server class that dispatches JSON-RPC requests
 * from a `Client` to a processor.
 */
export default abstract class Server {

  /**
   * The processor that this server dispatches to.
   */
  processor: Processor

  /**
   * The logging level.
   *
   * If undefined then no logging is done.
   * If greater than zero, all requests are logged.
   * If less than zero, only requests which result in an error
   * with a code less than the value are logged.
   * See the [JSON-RPC spec](https://www.jsonrpc.org/specification#error_object) for
   * error codes.
   */
  logging?: number

  constructor (processor: Processor = new Processor(), logging?: number) {
    this.processor = processor
    this.logging = logging
  }

  /**
   * Handle a JSON-RPC 2,0 request
   *
   * @param json A JSON-PRC request
   * @param stringify Should the response be stringified?
   * @returns A JSON-RPC response as an object or string (default)
   */
  recieve (request: string | JsonRpcRequest, stringify: boolean = true): string | JsonRpcResponse {
    const response = new JsonRpcResponse(-1)

    // Extract a parameter by name from Object or by index from Array
    // tslint:disable-next-line:completed-docs
    function param (request: JsonRpcRequest, index: number, name: string, required: boolean = true) {
      if (!request.params) throw new JsonRpcError(-32600, 'Invalid request: missing "params" property')
      const value = Array.isArray(request.params) ? request.params[index] : request.params[name]
      if (required && value === undefined) throw new JsonRpcError(-32602, `Invalid params: "${name}" is missing`)
      return value
    }

    try {
      if (typeof request === 'string') {
        // Parse JSON into an request
        try {
          request = JSON.parse(request) as JsonRpcRequest
        } catch (err) {
          throw new JsonRpcError(-32700, 'Parse error: ' + err.message)
        }
      }

      // Response id is same as the request id
      response.id = request.id

      if (!request.method) throw new JsonRpcError(-32600, 'Invalid request: missing "method" property')

      let result
      switch (request.method) {
        case 'manifest':
          result = this.processor.manifest()
          break
        case 'import':
          result = this.processor.import(
            param(request, 0, 'thing'),
            param(request, 1, 'format', false)
          )
          break
        case 'export':
          result = this.processor.export(
            param(request, 0, 'thing'),
            param(request, 1, 'format', false)
          )
          break
        case 'convert':
          result = this.processor.convert(
            param(request, 0, 'thing'),
            param(request, 1, 'from', false),
            param(request, 2, 'to', false)
          )
          break
        case 'compile':
          result = this.processor.compile(
            param(request, 0, 'thing'),
            param(request, 1, 'format', false)
          )
          break
        case 'build':
          result = this.processor.build(
            param(request, 0, 'thing'),
            param(request, 1, 'format', false)
          )
          break
        case 'execute':
          result = this.processor.execute(
            param(request, 0, 'thing'),
            param(request, 1, 'format', false)
          )
          break
        default:
          throw new JsonRpcError(-32601, `Method not found: "${request.method}"`)
      }

      // Most functions return a Thing tht needs to be exported to an Object
      // to include in the response JSON
      response.result = (result instanceof Thing) ? this.processor.exportObject(result) : result
    } catch (exc) {
      response.error = (exc instanceof JsonRpcError) ? exc : new JsonRpcError(-32603, `Internal error: ${exc.message}`, { trace: exc.stack })
    }

    if (this.logging !== undefined) {
      if (this.logging === 0 || (response.error && response.error.code <= this.logging)) {
        this.log({ request, response })
      }
    }

    return stringify ? JSON.stringify(response) : response
  }

  /**
   * Create a log entry
   *
   * Standard error is used since that is the standard stream that should be used
   * for "writing diagnostic output" according to the [POSIX standard](https://www.unix.com/man-page/POSIX/3posix/stderr/)
   *
   * @param entry The log entry. A timestamp is always added to this entry.
   */
  log (entry = {}) {
    if (typeof process !== 'undefined') { // tslint:disable-line:strict-type-predicates
      const timestamp = new Date().valueOf()
      entry = Object.assign({ timestamp }, entry)
      process.stderr.write(JSON.stringify(entry) + '\n')
    }
  }

  /**
   * Start the server
   */
  abstract start (): void

  /**
   * Stop the server
   */
  abstract stop (): void

  /**
   * Run the server with graceful shutdown on `SIGINT` or `SIGTERM`
   */
  run () {
    if (typeof process !== 'undefined') { // tslint:disable-line:strict-type-predicates
      const stop = () => this.stop()
      process.on('SIGINT', stop)
      process.on('SIGTERM', stop)
    }
    this.start()
  }
}
