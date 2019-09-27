import minimist from 'minimist'
import * as net from 'net'
import Executa from './executa'

const lps = require('length-prefixed-stream')
import { getLogger, LogLevel, replaceHandlers } from '@stencila/logga'

const log = getLogger('engine:serve')

const debug = true

replaceHandlers(data => {
  const { level, tag, message } = data
  if (level <= (debug ? LogLevel.debug : LogLevel.warn)) {
    process.stderr.write(
      `${tag}: ${LogLevel[level].toUpperCase()}: ${message}\n`
    )
  }
})

const { _, ...options } = minimist(process.argv.slice(2))

if (options.tcp !== undefined) {
  const server = net.createServer(async socket => {
    await main(socket, socket)
  })

  server.listen(options.tcp, '127.0.0.1')
  console.log('Listening')
}

function main(inStream: net.Socket, outStream: net.Socket): void {
  const executa = new Executa()
  try {
    const decode = lps.decode()
    inStream.pipe(decode)

    const encode = lps.encode()
    encode.pipe(outStream)

    decode.on('data', async (json: Buffer) => {
      const request = JSON.parse(json.toString())
      const { id, method, params } = request

      let error = null
      let result = null
      if (params === undefined || params.length !== 1) {
        error = 'params was not an array of length 1'
      } else {
        const node = params[0]
        result = await executa.execute(node)
      }

      const response = {
        jsonrpc: '2.0',
        id,
        result,
        error
      }
      encode.write(JSON.stringify(response))
    })
  } catch (err) {
    // log.write(err + '\n')
    console.error(err)
  }
}
