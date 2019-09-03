import express from 'express'
import bodyParser from 'body-parser'
import { getLogger, addHandler, LogData, LogLevel } from '@stencila/logga'
import browserSync from 'browser-sync'
import path from 'path'
// @ts-ignore
import Youch from 'youch'
import * as convert from './commands/convert'
import * as process from './commands/process'

const logger = getLogger('stencila')

/**
 * Web application interface
 *
 * @param folder The folder to serve
 * @param sync Automatically update the browser on changes to the folder?
 * @param port Port to listen on
 * @param address Address to listen on
 */
export default function web(
  folder: string,
  sync: boolean = false,
  port: number = 3000,
  address: string = 'localhost'
): void {
  const app = express()

  // Body parsing
  app.use(bodyParser.raw({ type: '*/*' }))

  // Add command endpoints
  convert.http(app, folder)
  process.http(app, folder)

  // Add error handling middleware to handle uncaught errors
  // and send them to the logger and client
  app.use(async (error: Error, req: express.Request, res: express.Response) => {
    const { message, stack } = error

    // Send error to logger
    logger.error({ message, stack })

    // Send a detailed error message as
    // HTML or JSON to the client
    res.status(500)
    if (req.accepts('html') !== undefined) {
      await htmlError(error, req).then((html: string) => {
        res.header('Content-Type', 'text/html')
        res.send(html)
      })
    } else {
      res.json({ message, stack })
    }
  })

  // Start server
  app.listen(port, address)
  const url = `http://${address}:${port}`
  logger.info(`Serving folder ${folder} at ${url}`)

  if (sync) {
    // Create a Browsersync server that proxies to server
    const browser = browserSync.create()
    browser.init({
      files: path.join(folder, '**', '*'),
      port: port + 1,
      proxy: {
        target: url
      },
      logPrefix: 'stencila'
    })
    // Add a log handler that displays errors in the browser
    addHandler((data: LogData) => {
      const html = `
      <div>
        <div>${LogLevel[data.level]}</div>
        <div>${data.message}</div>
        <pre>${data.level < 4 ? data.stack : ''}</pre>
      </div>`
      browser.notify(html, 30 * 1000)
    })
  }
}

function htmlError(error: Error, req: express.Request): Promise<string> {
  const youch = new Youch(error, req)
  return youch
    .addLink(({ message }: { message: string }) => {
      const url = `https://github.com/stencila/stencila/search?q=${encodeURIComponent(
        `${message.slice(0, 100)}`
      )}`
      return `<a href="${url}" target="_blank" title="Search on Github">
          <i class="fab fa-github"></i>
        </a>`
    })
    .addLink(({ message }: { message: string }) => {
      const url = `https://stackoverflow.com/search?q=${encodeURIComponent(
        `[stencila] ${message}`
      )}`
      return `<a href="${url}" target="_blank" title="Search on Stack Overflow">
          <i class="fab fa-stack-overflow"></i>
        </a>`
    })
    .toHTML()
}
