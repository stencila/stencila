// Run all tests (`*.test.js`)

var glob = require('glob');
var path = require('path');

glob.sync(path.join(__dirname, '/**/*.test.js')).forEach(function (pathname) {
  require(pathname);
});
