/**
 * Interfaces to the `execute` command.
 */

import express from 'express'
import yargs from 'yargs'
import * as dockta from '@stencila/dockta'
import asyncHandler from 'express-async-handler'

const DEFAULT_BUILD_TOOL = 'docker'

/**
 * Add `execute` CLI command to a `yargs` definition.
 *
 * @param yargsDefinition The current `yargs` definition that has been created in `cli.ts`.
 * @param callbackFunction Function to be called after the command has executed.
 */
export function cli(
  yargsDefinition: yargs.Argv,
  callbackFunction?: Function
): yargs.Argv {
  return yargsDefinition.command(
    'execute [folder] [command]',
    'Execute a command in the environment defined by the folder',
    cliArgsDefine,
    async (argv: any): Promise<void> => {
      await dockta.execute(argv.folder, argv.command, argv.using === 'nix')

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
      describe:
        'The path of the folder with the environment to execute the command in.',
      type: 'string'
    })
    .demand('folder')
    .positional('command', {
      describe: 'The command to run.',
      type: 'string'
    })
    .demand('command')
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
   * `POST /execute`: execute a command in the request's folder's environment
   */
  expressApp.post(
    '/execute',
    asyncHandler(async (req: express.Request, res: express.Response) => {
      const content = req.body || {}

      if (!content.folder) {
        throw new Error('Missing parameter "folder"')
      }

      if (!content.command) {
        throw new Error('Missing parameter "command"')
      }

      res.set('Content-Type', 'application/json')

      await dockta.execute(
        content.folder,
        content.command,
        content.using === 'nix',
        'json',
        res.send
      )
    })
  )
}
