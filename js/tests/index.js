// Require in all the test files
// Used by `npm test` and `npm run cover`

const glob = require('glob')
const path = require('path')

glob.sync(path.join(__dirname, '/**/*.test.js')).forEach(function (pathname) {
  require(pathname)
})
