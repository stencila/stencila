// Run one test file
// This module only exists for the babelize below
'use strict'

const path = require('path')
const args = process.argv.slice(2)

require('../babelize')
if (args.length) {
  let file = args[0]
  if (path.isAbsolute(file)) {
    file = path.relative(__dirname, file)
  } else {
    file = './' + file
  }
  console.log('Running test file: ' + file)
  require(file)
} else {
  throw Error('No test file specified in arguments')
}
