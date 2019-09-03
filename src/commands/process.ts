import * as encoda from '@stencila/encoda'
import encodaProcess from '@stencila/encoda/dist/process'
import { getLogger } from '@stencila/logga'
import express from 'express'
import asyncHandler from 'express-async-handler'
import fs from 'fs-extra'
import path from 'path'
import yargs from 'yargs'
import { cliArgsDefine } from './convert'
import { fallback } from '../util/fallback'

const logger = getLogger('stencila')

/**
 * Add CLI commands to a `yargs` definition.
 *
 * @param yargsDefinition The current `yargs` definition created in `cli.ts`.
 * @param next Function to be called after a command has executed.
 */
export function cli(yargsDefinition: yargs.Argv, next?: Function): void {
  yargsDefinition.command(
    'process [input] [output]',
    'Process content',
    cliArgsDefine,
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    async (argv: yargs.Arguments<any>): Promise<void> => {
      const { input, output, from, to } = argv

      const node = await encoda.read(input, from)
      const processed = await encodaProcess(node)
      await encoda.write(processed, output, to)

      if (next !== undefined) next()
    }
  )
}

/**
 * Add HTTP endpoints to an `express` application.
 *
 * @param expressApp The current `express` app created in `web.ts`
 * @param folder The folder
 */
export function http(expressApp: express.Application, folder: string): void {
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

      const mediaType = fallback(req.get('Accept'), 'text/html').split(/,|;/)[0]
      logger.info(`Processing ${file} to ${mediaType}`)

      const node = await encoda.read(filePath)
      const processed = await encodaProcess(node)
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
      const content = fallback(req.body, '')
      const mediaTypeFrom = fallback(
        req.get('Content-Type'),
        'application/json'
      )
      const mediaTypeTo = fallback(req.get('Accept'), 'application/json')
      logger.info(`Processing content from ${mediaTypeFrom} to ${mediaTypeTo}`)

      const node = await encoda.load(content, mediaTypeFrom)
      const processed = await encodaProcess(node)
      const result = await encoda.dump(processed, mediaTypeTo)

      res.set('Content-Type', mediaTypeTo)
      res.send(result)
    })
  )
}
