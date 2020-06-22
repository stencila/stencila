import yargs from 'yargs'
import { extractDeps } from '../boot'

/**
 * Add CLI arguments to a `yargs` definition.
 *
 * @param yargsDefinition The current `yargs` definition that has been created in `cli.ts`.
 * @param callbackFunction Function to be called after the command has executed.
 */
export function cli(
  yargsDefinition: yargs.Argv,
  callbackFunction?: Function
): yargs.Argv {
  return yargsDefinition.command(
    'setup',
    'Force re-extraction of the stencila CLI environment deps',
    (yargsDefinition: yargs.Argv): yargs.Argv => {
      return yargsDefinition
    },
    // TODO: Fix Linting errors
    // eslint-disable-next-line @typescript-eslint/no-misused-promises, @typescript-eslint/require-await
    async (): Promise<void> => {
      extractDeps(true)
      if (callbackFunction !== undefined) callbackFunction()
    }
  )
}
