/**
 * Interfaces to the `build` command.
 */

import express from 'express'
import yargs from 'yargs'
import * as dockta from '@stencila/dockta'
import asyncHandler from 'express-async-handler'

const DEFAULT_BUILD_TOOL = 'docker'

/**
 * Add `build` CLI command to a `yargs` definition.
 *
 * @param yargsDefinition The current `yargs` definition that has been created in `cli.ts`.
 * @param callbackFunction Function to be called after the command has executed.
 */
export function cli(
  yargsDefinition: yargs.Argv,
  callbackFunction?: Function
): yargs.Argv {
  return yargsDefinition.command(
    'build [folder]',
    'Build a folder into an environment',
    cliArgsDefine,
    async (argv: any): Promise<void> => {
      await dockta.build(argv.folder, argv.using === 'nix')

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
    .positional('folder', {
      describe: 'The path of the folder to build.',
      type: 'string'
    })
    .demand('folder')
    .option('using', {
      describe: `The tool to use ("docker" or "nix", default: ${DEFAULT_BUILD_TOOL}).`,
      type: 'string',
      default: DEFAULT_BUILD_TOOL
    })
}

/**
 * Add HTTP endpoints to an `express` application.
 *
 * @param expressApp The current `express` app created in `web.ts`
 * @param folder The folder
 */
export function http(expressApp: express.Application, folder: string) {
  /**
   * `POST /build`: build an environment from the folder in the request
   */
  expressApp.post(
    '/convert',
    asyncHandler(async (req: express.Request, res: express.Response) => {
      const content = req.body || {}

      if (!content.folder) {
        throw new Error('Missing parameter "folder"')
      }

      await dockta.build(content.folder, content.using === 'nix')

      res.send('Build maybe completed without errors.')
    })
  )
}
