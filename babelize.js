// Register the Babel require hook

require('babel-register')({
  // By default, `babel-register` ignores everything in `node-modules`
  // See https://babeljs.io/docs/usage/require/
  // Override that behaviour so that substance is transpiled
  ignore: function (filename) {
    if (filename.match('.*/node_modules/substance/.+')) {
      return false
    } else if (filename.match('.*/node_modules/.+')) {
      return true
    } else {
      return false
    }
  }
})
