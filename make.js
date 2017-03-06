/* globals __dirname */
var b = require('substance-bundler')
var path = require('path')
var fs = require('fs')

// this is not run all the time
// we use it to pre-bundle vendor libraries,
// to speed up bundling within this project
// and to work around problems with using rollup on these projects
function _buildVendor() {
  _minifiedVendor('./node_modules/sanitize-html/index.js', 'sanitize-html', {
    exports: ['default']
  })
  // ATTENTION: brace is exposing window.ace,
  // thus we need to use 'window.ace' when defining brace as 'external'
  _minifiedVendor('./.make/brace.js', 'brace')
}

function _minifiedVendor(src, name, opts = {}) {
  let tmp = `./vendor/${name}.js`
  let _opts = Object.assign({
    dest: tmp,
    debug: false
  })
  if (opts.exports) {
    _opts.exports = opts.exports
  }
  if (opts.standalone) {
    _opts.browserify = { standalone: name }
  }
  b.browserify(src, _opts)
  if (opts.minify !== false) {
    b.minify(tmp, {
      debug: false
    })
    b.rm(tmp)
  }
}

// we need this only temporarily, or if we need to work on an
// unpublished version of substance
function _buildSubstance() {
  if (!fs.existsSync(path.join(__dirname, 'node_modules/substance/dist/substance.js'))){
    b.make('substance', 'browser:pure')
  }
  if (!fs.existsSync(path.join(__dirname, 'node_modules/substance-expression/dist/substance-expression.js'))){
    b.make('substance-expression')
  }
}

function _copyAssets() {
  b.copy('./node_modules/font-awesome', './build/font-awesome')
  b.copy('./fonts', './build/fonts')
  b.copy('./vendor/brace.*', './build/web/')
  // We need to build to ROOT/katex because katex.min.js attempts to load
  // katex.min.css from /katex/katex.min.css
  b.copy('./node_modules/katex/dist/', './build/katex')
  b.copy('./node_modules/substance/dist/substance.js*', './build/web/')
  b.copy('./node_modules/substance-expression/dist/substance-expression.js*', './build/web/')
}

function _buildCss() {
  b.css('src/pagestyle/stencila.css', 'build/stencila.css', {
    variables: true
  })
}

function _buildDocument() {
  b.js('src/document/document.js', {
    dest: 'build/stencila-document.js',
    format: 'umd', moduleName: 'stencilaDocument',
    // Ignoring stencila-js for now because
    // it needs to be re-designed to be really browser compatible
    alias: {
      'stencila-js': path.join(__dirname, 'vendor/stencila-js.stub.js'),
      // 'brace': path.join(__dirname, 'vendor/brace.min.js'),
      'sanitize-html': path.join(__dirname, 'vendor/sanitize-html.min.js'),
    },
    // TODO: here we need to apply different strategies for
    // different bundles (e.g. hosted without substance, but electron one with substance)
    external: {
      'substance': 'window.substance',
      // brace bundle is exposing window.ace
      'brace': 'window.ace',
      'katex': 'window.katex'
    },
    commonjs: true,
    json: true
  })
}

function _buildSheet() {
  b.js('src/sheet/sheet.js', {
    dest: 'build/stencila-sheet.js',
    format: 'umd', moduleName: 'stencilaSheet',
    // Ignoring stencila-js for now because
    // it needs to be re-designed to be really browser compatible
    alias: {
      'stencila-js': path.join(__dirname, 'vendor/stencila-js.stub.js')
    },
    external: {
      'substance': 'substance',
      'substance-expression': 'substanceExpression'
    },
    commonjs: true,
    json: true
  })
}

function _buildExamples() {
  b.copy('./examples/*/*.html', './build/')
  b.js('examples/document/app.js', {
    dest: 'build/examples/document/app.js',
    format: 'umd', moduleName: 'documentExample',
    external: {
      'substance': 'substance',
      'stencila-document': 'stencilaDocument'
    }
  })
  b.js('examples/sheet/app.js', {
    dest: 'build/examples/sheet/app.js',
    format: 'umd', moduleName: 'sheetExample',
    external: {
      'substance': 'substance',
      'stencila-sheet': 'stencilaSheet'
    }
  })
}

b.task('clean', () => {
  b.rm('build')
})

// This is used to generate the files in ./vendor/
b.task('vendor', _buildVendor)

// ATTENTION: this is not necessary after switching to a published version of substance
b.task('substance', () => {
  _buildSubstance()
})

b.task('assets', ['substance'], () => {
  _copyAssets()
})

b.task('css', ['substance'], () => {
  _buildCss()
})

b.task('document', ['assets', 'css'], () => {
  _buildDocument()
})

b.task('sheet', ['assets', 'css'], () => {
  _buildSheet()
})

b.task('examples', ['clean', 'assets', 'css', 'document', 'sheet'], () => {
  _buildExamples()
})

b.task('default', ['clean', 'assets', 'examples'])

b.serve({ static: true, route: '/', folder: 'build' })
