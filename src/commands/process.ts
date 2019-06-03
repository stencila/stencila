import express from 'express'
import asyncHandler from 'express-async-handler'
import yargs from 'yargs'
import * as encoda from '@stencila/encoda'
import { cliArgsDefine, cliArgsDefaults } from './convert'
import path from 'path'
import { getLogger } from '@stencila/logga'
import fs from 'fs-extra'

const logger = getLogger('stencila')

/**
 * Add CLI commands to a `yargs` definition.
 *
 * @param yargsDefinition The current `yargs` definition created in `cli.ts`.
 * @param next Function to be called after a command has executed.
 */
export function cli(yargsDefinition: yargs.Argv, next?: Function) {
  yargsDefinition.command(
    'process [input] [output]',
    'Process content',
    cliArgsDefine,
    async (argv: any): Promise<void> => {
      const { input, output, from, to } = cliArgsDefaults(argv)

      const node = await encoda.read(input, from)
      const processed = await encoda.process(node)
      await encoda.write(processed, output, to)

      if (next) next()
    }
  )
}

/**
 * Add HTTP endpoints to an `express` application.
 *
 * @param expressApp The current `express` app created in `web.ts`
 * @param folder The folder
 */
export function http(expressApp: express.Application, folder: string) {
  /**
   * `GET /<file>`: process a file from the currently served folder
   *
   * This endpoint is intended browsers and returns HTML content
   * by default but will also return other content (e.g. JSON)
   * specified in the `Accept` header.
   */
  expressApp.get(
    '/*',
    asyncHandler(async (req: express.Request, res: express.Response) => {
      const file = req.path.slice(1)
      const filePath = path.join(folder, file)
      if (!(await fs.pathExists(filePath))) {
        return res.sendStatus(404)
      }

      const mediaType = (req.get('Accept') || 'text/html').split(/,|;/)[0]
      logger.info(`Processing ${file} to ${mediaType}`)

      const node = await encoda.read(filePath)
      const processed = await encoda.process(node)
      const content = await encoda.dump(processed, mediaType)

      res.set('Content-Type', mediaType)
      res.send(content)
    })
  )

  /**
   * `POST /process`: process content in the request body
   */
  expressApp.post(
    '/process',
    asyncHandler(async (req: express.Request, res: express.Response) => {
      const content = req.body || {}
      const mediaTypeFrom = req.get('Content-Type') || 'application/json'
      const mediaTypeTo = req.get('Accept') || 'application/json'
      logger.info(`Processing content from ${mediaTypeFrom} to ${mediaTypeTo}`)

      const node = await encoda.load(content, mediaTypeFrom)
      const processed = await encoda.process(node)
      const result = await encoda.dump(processed, mediaTypeTo)

      res.set('Content-Type', mediaTypeTo)
      res.send(result)
    })
  )
}
