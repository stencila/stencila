// Run functional tests (`*.fun.js`)

// Require so THIS file does not need a babelize before it
// (allows tests to be run without babel-node)
const glob = require('glob')
const path = require('path')
const test = require('tape')

require('../babelize')
require('../server')

// Require all functional tests
glob.sync(path.join(__dirname, '/**/*.fun.js')).forEach(function (pathname) {
  require(pathname)
})

// Exit the process when all tests have finished running
// (otherwise server keeps on servin`)
test.onFinish(function () {
  process.exit()
})
