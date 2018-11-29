import { Readable, Writable } from 'stream'
import * as readline from 'readline'

import Server from './Server'
import Processor from '../Processor'

/**
 * A `Server` using standard input/output for communication.
 */
export default class StdioServer extends Server {

  /**
   * The stream that requests are recieved on
   */
  input: Readable

  /**
   * The stream that responses are sent on
   */
  output: Writable

  /**
   * A `readline` interface
   */
  io?: readline.ReadLine

  constructor (processor?: Processor, logging?: number, input: Readable = process.stdin, output: Writable = process.stdout) {
    super(processor, logging)

    this.input = input
    this.output = output
  }

  // Method overriden from `Server`

  start () {
    this.io = readline.createInterface({
      input: this.input,
      output: this.output,
      prompt: ''
    }).on('line', request => {
      this.output.write(this.recieve(request) + '\n')
    })
    this.log({ started: true })
  }

  stop () {
    if (this.io) {
      this.io.close()
      this.io = undefined
      this.log({ stopped: true })
    }
  }
}
