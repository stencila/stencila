// Register the Babel require hook
// Used in various entry points to automatically run Babel transpilation
// on `require`d modules

require('babel-register')({
  // es2015 preset for `import` and `export` support
  presets: [ 'es2015' ],
  // By default, `babel-register` ignores everything in `node-modules`
  // See https://babeljs.io/docs/usage/require/
  // Override that behaviour so that substance is transpiled
  ignore: function (filename) {
    if (filename.match('.*/substance/.+')) {
      return false
    } else if (filename.match('.*/node_modules/.+')) {
      return true
    } else {
      return false
    }
  }
})
