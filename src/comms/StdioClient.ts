import { Readable, Writable } from 'stream'
import * as readline from 'readline'

import Client from './Client'
import JsonRpcRequest from './JsonRpcRequest'

/**
 * A `Client` using standard input/output
 * for communication.
 */
export default class StdioClient extends Client {

  /**
   * The stream that responses are recieved on
   */
  input: Readable

  /**
   * The stream that requests are sent on
   */
  output: Writable

  /**
   * A `readline` interface
   */
  io: readline.ReadLine

  /**
   * Constructor
   *
   * @param stdin The standard input of the `StdioServer`
   * @param stdout The standard output of the `StdioServer`
   */
  constructor (input: Readable = process.stdin, output: Writable = process.stdout) {
    super()

    this.input = input
    this.output = output

    this.io = readline.createInterface({
      input: this.input,
      output: this.output,
      prompt: ''
    }).on('line', response => {
      this.recieve(response)
    })
  }

  // Overrides of `Client` methods

  send (request: JsonRpcRequest) {
    this.output.write(JSON.stringify(request) + '\n')
  }
}
