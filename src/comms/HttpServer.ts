import express from 'express'
import getPort from 'get-port'
import http from 'http'

import Server from './Server'
import Processor from '../Processor'
import JsonRpcRequest from './JsonRpcRequest'
import JsonRpcResponse from './JsonRpcResponse'

/**
 * A `Server` using HTTP for communication.
 */
export default class HttpServer extends Server {

  /**
   * The port to listen on.
   */
  port: number

  /**
   * The address to listen on.
   *
   * Usually 127.0.0.1 or 0.0.0.0.
   */
  address: string

  /**
   * The Express application
   *
   * Used by derived classes to add routes.
   */
  app: express.Application

  /**
   * The `http` module server
   */
  server?: http.Server

  constructor (processor?: Processor, logging?: number, port: number = 2000, address: string = '127.0.0.1') {
    super(processor, logging)

    this.port = port
    this.address = address

    const app = express()

    app.disable('x-powered-by')
    app.use(express.json())

    // JSON-RPC over HTTP
    // There is no wrapping/unwrapping of the request/response or
    // special handling of errors
    app.post('/', (req, res) => {
      res.setHeader('Content-Type', 'application/json')
      res.send(this.recieve(req.body))
    })

    // JSON-RPC wrapped in HTTP
    // Unrap the HTTP request into a JSON RPC request and
    // wrap the response as HTTP
    const wrap = (method: string) => {
      return (req: express.Request, res: express.Response) => {
        const request = new JsonRpcRequest(method, req.body)
        const response = this.recieve(request, false) as JsonRpcResponse
        // For export and convert methods the content type may be some other format
        res.setHeader('Content-Type', 'application/json')
        if (response.error !== undefined) res.status((response.error.code < -32603) ? 400 : 500).send({ error: response.error })
        else res.send(response.result)
      }
    }
    app.post('/import', wrap('import'))
    app.post('/export', wrap('export'))
    app.post('/convert', wrap('convert'))
    app.post('/compile', wrap('compile'))
    app.post('/build', wrap('build'))
    app.post('/execute', wrap('execute'))

    // TODO: work out how to set this folder, if at all
    app.use('/', express.static('./tests/comms'))

    this.app = app
  }

  // Methods overriden from `Server`

  async start () {
    this.port = await getPort({ port: this.port }) // tslint:disable-line:await-promise
    this.server = this.app.listen(this.port, this.address, () => {
      this.log({ started: `http://${this.address}:${this.port}` })
    })
  }

  stop () {
    if (this.server) {
      this.server.close()
      this.server = undefined
      this.log({ stopped: true })
    }
  }
}
