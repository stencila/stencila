#!/usr/bin/env node

import * as encoda from '@stencila/encoda'
import * as logga from '@stencila/logga'
import fs from 'fs'
import path from 'path'
import tar from 'tar'
import yargs from 'yargs'

import pkg from '../package.json'

const log = logga.getLogger('stencila')

const args = yargs
  .scriptName('stencila')
  // Ensure that a command is provided
  .demandCommand(1, 'Please provide a command.')

  // Any command-line argument given that is not demanded,
  // or does not have a corresponding description, will be
  // reported as an error.
  .strict()

  // Maximize width of usage instructions
  .wrap(yargs.terminalWidth())

  // Help global option
  .usage('$0 <cmd> [args]')
  .alias('help', 'h')

  // Version global option
  .version(pkg.version)
  .alias('version', 'v')
  .describe('version', 'Show version')

  // Debug global option
  .option('debug', {
    description: 'Show debug level log entries',
    default: false,
    type: 'boolean',
  })

  // Setup command
  .command(
    'setup',
    'Set up the application',
    (yargs: yargs.Argv) => {
      yargs.option('force', {
        describe: 'Force reinstall of dependencies?',
        default: false,
        type: 'boolean',
      })
    },
    setup
  )

  // Convert command
  .command(
    'convert input [outputs..]',
    'Convert between file formats',
    (yargs: yargs.Argv) => {
      yargs
        .positional('input', {
          describe: 'The input file path. Defaults to standard input.',
          type: 'string',
          default: '-',
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
          describe: `The theme to use for the output format.`,
          type: 'string',
          default: 'stencila',
        })
        .option('zip', {
          describe:
            'Create Zip archive containing output files? no (default), yes, maybe (only if more than one file)',
          choices: ['no', 'yes', 'maybe'],
          default: 'no',
        })
    },
    convert
  )

  // Unhandled errors
  .fail(function (msg: string, err: Error) {
    if (err !== undefined) log.error(err)
    else log.error(msg)
    process.exit(1)
  }).argv

/**
 * Configure log event handling
 */
logga.replaceHandlers((data: logga.LogData): void => {
  logga.defaultHandler(data, {
    maxLevel: (args.debug as boolean)
      ? logga.LogLevel.debug
      : logga.LogLevel.info,
    throttle: {
      // Do not repeat the same message within 5s
      signature: `${data.tag}${data.level}${data.message}`,
      duration: 5000,
    },
  })
})

/**
 * Setup the application
 *
 * The [`pkg`](https://github.com/zeit/pkg) Node.js packager does not
 * package native modules.  i.e `*.node` files. There are various ways to handle this but
 * we found the easiest/safest was to simply copy the directories for the
 * packages with native modules, from the host system, into directory where the
 * binary is installed. This function does that via `encoda-deps.tar.gz` which is
 * packaged in the binary snapshot as an `asset`.
 *
 * See:
 *   - https://github.com/stencila/encoda/pull/47#issuecomment-489912132
 *   - https://github.com/zeit/pkg/issues/329
 *   - https://github.com/JoshuaWise/better-sqlite3/issues/173
 *   - `package.json`
 */
function setup(
  argv: yargs.Arguments<{
    force: boolean
  }>
): void {
  // Is this process being run as a `pkg` packaged binary?
  const packaged =
    (process.mainModule?.id.endsWith('.exe') ||
      Object.prototype.hasOwnProperty.call(process, 'pkg')) &&
    fs.existsSync(path.join('/', 'snapshot'))

  /**
   * The home directory for this modules or process where
   * native modules and executables are placed.
   */
  const home = packaged
    ? path.dirname(process.execPath)
    : path.dirname(__dirname)

  const shouldExtract =
    packaged && (argv.force || !fs.existsSync(path.join(home, 'node_modules')))
  if (shouldExtract) {
    tar.x({
      sync: true,
      file: path.join('/', 'snapshot', 'stencila', 'stencila-deps.tgz'),
      strip: 1,
      C: home,
    })

    log.info('Setup complete.')
  }
}

// Run setup every time
setup({ force: false, _: [], $0: '' })

/**
 * The `convert` command.
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
function convert(
  argv: yargs.Arguments<{
    input: string
    outputs: string[]
    from: string
    to: string
    theme?: string
    standalone: boolean
    zip: 'yes' | 'no' | 'maybe'
  }>
): void {
  let { input, outputs, from, to, theme, zip } = argv

  if (input === '-' && from === undefined) from = 'md'
  if (outputs === ['-'] && to === undefined) to = 'yaml'

  encoda
    .convert(input, outputs, {
      from,
      to,
      encodeOptions: {
        theme,
        isStandalone: true,
        isBundle: false,
        shouldZip: zip,
      },
    })
    .catch((error) => log.error(error))
}
