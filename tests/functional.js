// Run functional tests (`*.fun.js`)

require('../babelize');

import glob from 'glob'
import path from 'path'
import test from 'tape'

require('../server');

// Require all functional tests
glob.sync(path.join(__dirname, '/**/*.fun.js')).forEach(function (pathname) {
  require(pathname);
});

// Exit the process when all tests have finished running
// (otherwise server keeps on servin`)
test.onFinish(function () {
  process.exit();
});
