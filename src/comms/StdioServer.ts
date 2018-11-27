import * as readline from 'readline'

import Server from './Server'

/**
 * A `Server` using standard input/output
 * for communication.
 */
export default class StdioServer extends Server {

  /**
   * A stdin/stdout `readline` interface
   */
  stdio?: readline.ReadLine

  // Method overriden from `Server`

  start () {
    this.stdio = readline.createInterface({
      input: process.stdin,
      output: process.stdout,
      prompt: ''
    }).on('line', request => {
      process.stdout.write(this.recieve(request) + '\n')
    })
  }

  stop () {
    if (this.stdio) {
      this.stdio.close()
      this.stdio = undefined
    }
  }
}
