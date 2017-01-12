// Register the Babel require hook
// Used in various entry points to automatically run Babel transpilation
// on `require`d modules

require('babel-register')({
  // es2015 preset for `import` and `export` support
  // Using `require` here seems to fix "Couldn't find preset "es2015" relative to directory"
  // bug (https://github.com/laravel/elixir/issues/354#issuecomment-251304711)
  presets: [ require('babel-preset-es2015') ],
  // By default, `babel-register` ignores everything in `node_modules`
  // See https://babeljs.io/docs/usage/require/
  // Override that behaviour so that substance and other modules that need to be transpiled, are
  ignore: function (filename) {
    if (filename.match('.*/node_modules/(substance|lodash-es|stencila-js)/.+')) {
      return false
    } else if (filename.match('.*/node_modules/.+')) {
      return true
    } else {
      return false
    }
  }
})
