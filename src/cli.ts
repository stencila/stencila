#!/usr/bin/env node

/**
 * Command line interface (CLI)
 *
 * This modules defines the `stencila` command line
 * interface to commands.
 */

import yargs from 'yargs'
import * as convert from './commands/convert'
import * as serve from './commands/serve'
import * as system from './commands/system'
import * as boot from './boot'
import * as log from './log'

const VERSION = require('../package').version

log.configure()
boot.extractDeps()

const yargsDefinition = yargs.scriptName('stencila')

// Add commands
convert.cli(yargsDefinition, cleanup)
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

  // Unhandled errors
  .fail(function(msg, err) {
    if (err !== undefined) log.logger.error(err)
    else log.logger.error(msg)
    process.exit(1)
  })

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
