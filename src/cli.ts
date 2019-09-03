#!/usr/bin/env node

/**
 * Command line interface (CLI)
 *
 * This modules defines the `stencila` command line
 * interface to commands.
 */

import yargs from 'yargs'
// @ts-ignore
import Youch from 'youch'
// @ts-ignore
import youchTerminal from 'youch-terminal'
import { addHandler, LogData, LogLevel } from '@stencila/logga'

import { setupLogger } from './logs'
import * as convert from './commands/convert'
import * as process_ from './commands/process'
import * as serve from './commands/serve'
import * as system from './commands/system'
import { extractDeps } from './boot'

const VERSION = require('../package').version

const winstonLogger = setupLogger()

// Add handler to send log events to winston
addHandler(function(data: LogData) {
  if (data.level <= LogLevel.error) {
    const youch = new Youch({ message: data.message, stack: data.stack }, {})
    youch.toJSON().then((obj: unknown) => console.error(youchTerminal(obj)))
  }

  winstonLogger.log(
    LogLevel[data.level],
    data.message,
    // Only record stack traces for errors and worse.
    data.level <= LogLevel.error ? data.stack : undefined
  )
})

/**
 * Attempt to extract the dependencies, nothing will happen if they have already been extracted
 */
extractDeps()

const yargsDefinition = yargs.scriptName('stencila')

// Add commands
convert.cli(yargsDefinition, cleanup)
process_.cli(yargsDefinition, cleanup)
serve.cli(yargsDefinition, cleanup)
system.cli(yargsDefinition, cleanup)

// Add yargs options and parse the args
yargsDefinition
  // Ensure that a command is provided
  .demandCommand(1, 'Please provide a command.')

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

// Clean up before process.exit
function cleanup(): void {
  // Emit a beforeExit event. e.g. used by Encoda's Puppeteer interface to
  // destroy any browser instance. Note that:
  //   "The 'beforeExit' event is not emitted for conditions causing
  //   explicit termination, such as calling process.exit() or uncaught
  //   exceptions."
  process.emit('beforeExit', 0)
}
