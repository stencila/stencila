import * as readline from 'readline'

import Client from './Client'

/**
 * A `Client` using standard input/output
 * for communication.
 */
export default class StdioClient extends Client {

  /**
   * A stdin/stdout `readline` interface
   */
  stdio: readline.ReadLine

  /**
   * The stream that requests are sent on
   */
  output: NodeJS.WritableStream

  /**
   * Constructor
   *
   * @param stdin The standard input of the `StdioServer`
   * @param stdout The standard output of the `StdioServer`
   */
  constructor (stdin: NodeJS.WritableStream, stdout: NodeJS.ReadableStream) {
    super()

    // Note that the stdout of the server is the input
    // and the stdin of the server is the output!
    this.output = stdin
    this.stdio = readline.createInterface({
      input: stdout,
      output: stdin,
      prompt: ''
    }).on('line', response => {
      this.recieve(response)
    })
  }

  // Overrides of `Client` methods

  send (request: string) {
    this.output.write(request + '\n')
  }
}
