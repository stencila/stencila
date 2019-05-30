import yargs from 'yargs'
import encoda from '@stencila/encoda'

const VERSION = require('../package').version

yargs
  .scriptName('stencila')

  .command(
    'convert [in] [out]',
    'Convert between file formats',
    (yargs: yargs.Argv): void => {
      yargs
        .positional('in', {
          describe: 'The input file path. Defaults to standard input.',
          type: 'string',
          default: '-'
        })
        .positional('out', {
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
    async (argv: yargs.Argv): Promise<void> => {
      const inp = argv.in
      const out = argv.out
      const from = argv.from
      const to = argv.to
      await encoda.convert(inp, out, { from, to })

      // Trigger a clean up
      //   "The 'beforeExit' event is not emitted for conditions causing
      //   explicit termination, such as calling process.exit() or uncaught
      //   exceptions."
      process.emit('beforeExit', 0)
    }
  )

  // Any command-line argument given that is not demanded, or does not have a corresponding description, will be reported as an error.
  // Unrecognized commands will also be reported as errors.
  .strict()

  // Maximize width of usage instructions
  .wrap(yargs.terminalWidth())

  // Help global option
  .usage('$0 <cmd> [args]')
  .alias('help', 'h')

  // Version global option
  .version(VERSION)
  .alias('version', 'v')
  .describe('version', 'Show version')

  .parse()
