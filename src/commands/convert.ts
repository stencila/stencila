/**
 * Interfaces to the `convert` command.
 */

import express from 'express'
import yargs from 'yargs'
import * as encoda from '@stencila/encoda'
import asyncHandler from 'express-async-handler'
import { getLogger } from '@stencila/logga'

const logger = getLogger('stencila')

/**
 * Add `convert` CLI command to a `yargs` definition.
 *
 * @param yargsDefinition The current `yargs` definition that has been created in `cli.ts`.
 * @param callbackFunction Function to be called after the command has executed.
 */
export function cli(
  yargsDefinition: yargs.Argv,
  callbackFunction?: Function
): yargs.Argv {
  return yargsDefinition.command(
    'convert [input] [output]',
    'Convert between file formats',
    cliArgsDefine,
    async (argv: any): Promise<void> => {
      const { input, output, from, to } = cliArgsDefaults(argv)
      await encoda.convert(input, output, { from, to })
      if (callbackFunction) callbackFunction()
    }
  )
}

/**
 * Add CLI arguments to a `yargs` definition.
 *
 * This function has been factored out to allow for reuse for other
 * commands with the same, or similar arguments.
 *
 * @param yargsDefinition The current `yargs` definition that has been created in `cli.ts`.
 */
export function cliArgsDefine(yargsDefinition: yargs.Argv): yargs.Argv<any> {
  return yargsDefinition
    .positional('input', {
      describe: 'The input file path. Defaults to standard input.',
      type: 'string',
      default: '-'
    })
    .positional('output', {
      describe: 'The output file path. Defaults to standard output.',
      type: 'string',
      default: '-'
    })
    .option('from', {
      describe: 'The format to convert the input from.',
      type: 'string'
    })
    .option('to', {
      describe: 'The format to convert the output to.',
      type: 'string'
    })
}

/**
 * Generate defaults for CLI arguments based on other arguments.
 *
 * For standard input, or content that does not appear to be a file with an extension,
 * default to Markdown input as the most human writable format
 * in the terminal.
 *
 * For standard output, default to YAML output as the most human readable format
 * in the terminal.
 *
 * @param argv Array of arguments
 */
export function cliArgsDefaults(argv: any) {
  let { input, output, from, to } = argv
  if ((input === '-' || !/\.([a-z]{2,5})$/.test(input)) && !from) from = 'md'
  if (output === '-' && !to) to = 'yaml'
  return { input, output, from, to }
}

/**
 * Add HTTP endpoints to an `express` application.
 *
 * @param expressApp The current `express` app created in `web.ts`
 * @param folder The folder
 */
export function http(expressApp: express.Application, folder: string) {
  /**
   * `POST /convert`: convert content in the request body
   */
  expressApp.post(
    '/convert',
    asyncHandler(async (req: express.Request, res: express.Response) => {
      const content = req.body || {}
      const mediaTypeFrom = req.get('Content-Type') || 'application/json'
      const mediaTypeTo = req.get('Accept') || 'application/json'
      logger.info(`Converting content from ${mediaTypeFrom} to ${mediaTypeTo}`)

      const node = await encoda.load(content, mediaTypeFrom)
      const result = await encoda.dump(node, { format: mediaTypeTo })

      res.set('Content-Type', mediaTypeTo)
      res.send(result)
    })
  )
}
