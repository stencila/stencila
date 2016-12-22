const glob = require('glob')
const path = require('path')

// Require in all the test files
glob.sync(path.join(__dirname, '/**/*.test.js')).forEach(function (pathname) {
  require(pathname)
})
