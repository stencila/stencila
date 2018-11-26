import JsonRpcError from './JsonRpcError'
import JsonRpcRequest from './JsonRpcRequest'
import JsonRpcResponse from './JsonRpcResponse'
import Processor from '../Processor'
import Thing from '../types/Thing'

/**
 * A base server class that dispatches JSON-RPC requests
 * from a `Client` to a processor.
 *
 * Standard error is used since that is the standard stream that should be used
 * for "writing diagnostic output" according to the [POSIX standard](https://www.unix.com/man-page/POSIX/3posix/stderr/)
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
   * @returns A JSON-RPC response
   */
  handle (json: string): string {
    let request: JsonRpcRequest = new JsonRpcRequest()
    const response = new JsonRpcResponse()

    // Extract a parameter by name from Object or by index from Array
    // tslint:disable-next-line:completed-docs
    function param (request: JsonRpcRequest, index: number, name: string, required: boolean = true) {
      if (!request.params) throw new JsonRpcError(-32600, 'Invalid request: missing "params" property')
      const value = Array.isArray(request.params) ? request.params[index] : request.params[name]
      if (required && value === undefined) throw new JsonRpcError(-32602, `Invalid params: "${name}" is missing`)
      return value
    }

    try {
      // Parse JSON into an request
      try {
        request = JSON.parse(json)
      } catch (err) {
        throw new JsonRpcError(-32700, 'Parse error: ' + err.message)
      }

      // Response must always have an id
      response.id = request.id || null

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
      response.error = (exc instanceof JsonRpcError) ? exc : new JsonRpcError(-32603, `Internal error: ${exc.message}`)
    }

    if (this.logging !== undefined) {
      if (this.logging === 0 || (response.error && response.error.code <= this.logging)) {
        const entry = this.log(request, response)
        process.stderr.write(JSON.stringify(entry) + '\n')
      }
    }

    return JSON.stringify(response)
  }

  /**
   * Create a log entry
   */
  log (request: JsonRpcRequest, response: JsonRpcResponse) {
    const timestamp = new Date().valueOf()
    return { timestamp, request, response }
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
    process.on('SIGINT', () => this.stop())
    process.on('SIGTERM', () => this.stop())
    this.start()
  }
}
