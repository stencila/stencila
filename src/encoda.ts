import yargs from 'yargs'
import * as encoda from '@stencila/encoda'

/**
 * Add CLI parsing of commands for the `encoda` library.
 *
 * @param yargsDefinition The current `yargs` definition that has been created in `cli.ts`.
 *                        Will be appended to with `encoda` specific commands.
 * @param cleanupFn function to be called after the command has executed
 */
export function addCliCommands(
  yargsDefinition: yargs.Argv,
  cleanupFn: Function
): yargs.Argv {
  return yargsDefinition.command(
    'convert [input] [output]',
    'Convert between file formats',
    (args: yargs.Argv<{}>): yargs.Argv<any> => {
      return args
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
    },
    async (argv: any): Promise<void> => {
      const { input, output, ...options } = argv
      await encoda.convert(input, output, options)
      cleanupFn()
    }
  )
}
