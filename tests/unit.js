// Run unit tests (`*.test.js`)

// Require so THIS file does not need a babelize before it
// (allows tests to be run without babel-node)
const glob = require('glob')
const path = require('path')

require('../babelize')

glob.sync(path.join(__dirname, '/**/*.test.js')).forEach(function (pathname) {
  require(pathname)
})
