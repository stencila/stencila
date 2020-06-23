/**
 * Interfaces to the `convert` command.
 */

import express from 'express'
import yargs from 'yargs'
import * as encoda from '@stencila/encoda'
import asyncHandler from 'express-async-handler'
import { getLogger } from '@stencila/logga'
import { fallback } from '../util/fallback'

const logger = getLogger('stencila')
const DEFAULT_THEME = 'stencila'

/**
 * Add `convert` CLI command to a `yargs` definition.
 *
 * Generate defaults for CLI arguments based on other arguments.
 *
 * For standard input, or content that does not appear to be a file with an extension,
 * default to Markdown input as the most human writable format
 * in the terminal.
 *
 * For standard output, default to YAML output as the most human readable format
 * in the terminal.
 *
 * @param yargsDefinition The current `yargs` definition that has been created in `cli.ts`.
 * @param callbackFunction Function to be called after the command has executed.
 */
export function cli(
  yargsDefinition: yargs.Argv,
  callbackFunction?: Function
): yargs.Argv {
  return yargsDefinition.command(
    'convert input [outputs..]',
    'Convert between file formats',
    cliArgsDefine,
    // eslint-disable-next-line @typescript-eslint/no-misused-promises
    async (
      argv: yargs.Arguments<{
        input: string
        outputs: string[]
        from: string
        to: string
      }>
    ): Promise<void> => {
      let { input, outputs, from, to, theme, zip } = argv
      if (input === '-' && from === undefined) from = 'md'
      if (outputs === ['-'] && to === undefined) to = 'yaml'
      await encoda.convert(
        input,
        outputs,
        // @ts-ignore
        {
          from,
          to,
          encodeOptions: {
            // TODO: Add type guards to avoid type casting
            theme: theme as string,
            shouldZip: zip as 'yes' | 'no' | 'maybe',
          },
        }
      )
      if (callbackFunction !== undefined) callbackFunction()
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
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function cliArgsDefine(yargsDefinition: yargs.Argv): yargs.Argv<any> {
  return yargsDefinition
    .positional('input', {
      describe: 'The input file path. Defaults to standard input.',
      type: 'string',
    })
    .positional('outputs', {
      describe: 'The output file path/s. Defaults to standard output.',
      type: 'string',
      default: '-',
    })
    .option('from', {
      describe: 'The format to convert the input from.',
      type: 'string',
    })
    .option('to', {
      describe: 'The format to convert the output to.',
      type: 'string',
    })
    .option('theme', {
      describe: `The theme to use for the output format (default: ${DEFAULT_THEME}).`,
      type: 'string',
      default: DEFAULT_THEME,
    })
    .option('zip', {
      describe:
        'Create Zip archive containing output files? no (default), yes, maybe (only if more than one file)',
      choices: ['no', 'yes', 'maybe'],
      default: 'no',
    })
}

/**
 * Add HTTP endpoints to an `express` application.
 *
 * @param expressApp The current `express` app created in `web.ts`
 * @param folder The folder
 */
export function http(expressApp: express.Application): void {
  /**
   * `POST /convert`: convert content in the request body
   */
  expressApp.post(
    '/convert',
    asyncHandler(async (req: express.Request, res: express.Response) => {
      const content = fallback(req.body, '')
      const mediaTypeFrom = fallback(
        req.get('Content-Type'),
        'application/json'
      )
      const mediaTypeTo = fallback(req.get('Accept'), 'application/json')
      const theme = fallback(req.query.theme, DEFAULT_THEME)
      logger.info(`Converting content from ${mediaTypeFrom} to ${mediaTypeTo}`)

      const node = await encoda.load(content, mediaTypeFrom)
      const result = await encoda.dump(node, mediaTypeTo, { theme })

      res.set('Content-Type', mediaTypeTo)
      res.send(result)
    })
  )
}
