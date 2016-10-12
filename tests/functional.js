// Run functional tests (`*.fun.js`)
'use strict'

// Requires (instead of imports) so that THIS file does not need a babelize before it
// (allows tests to be run without babel-node)
const glob = require('glob')
const path = require('path')

require('../babelize')

// Require all functional tests
glob.sync(path.join(__dirname, '/**/*.fun.js')).forEach(function (pathname) {
  require(pathname)
})
