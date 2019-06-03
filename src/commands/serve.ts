import yargs from 'yargs'
import web from '../web'

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
    'serve [folder]',
    'Serve a project folder',
    (yargsDefinition: yargs.Argv): yargs.Argv<any> => {
      return yargsDefinition
        .positional('folder', {
          describe:
            'The folder or file to preview. Defaults to current directory.',
          type: 'string',
          default: '.'
        })
        .option('sync', {
          describe: 'Synchonize the browser with changes in the folder?',
          type: 'boolean',
          default: false
        })
    },
    async (argv: any): Promise<void> => {
      let { folder, sync, include, exclude } = argv
      web(folder, sync)
      if (callbackFunction) callbackFunction()
    }
  )
}
