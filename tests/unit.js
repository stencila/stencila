// Run unit tests (`*.test.js`)

require('../babelize')

import glob from 'glob'
import path from 'path'

glob.sync(path.join(__dirname, '/**/*.test.js')).forEach(function (pathname) {
  require(pathname)
})
