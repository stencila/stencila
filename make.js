/* globals __dirname */
var b = require('substance-bundler')
var path = require('path')
var fs = require('fs')

const LIB_EXTERNAL = {
  'stencila-js': 'window.stencilaJs',
  'substance': 'window.substance',
  'substance-mini': 'window.substanceMini',
  // brace bundle is exposing window.ace
  'brace': 'window.ace',
  'katex': 'window.katex'
}

const EXAMPLE_EXTERNAL = {
  'substance': 'substance',
  'stencila-js': 'stencilaJs',
  'stencila-document': 'stencilaDocument',
  'stencila-sheet': 'stencilaSheet',
  'stencila': 'stencila'
}

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
function _buildDeps() {
  if (!fs.existsSync(path.join(__dirname, 'node_modules/substance/dist/substance.js'))){
    b.make('substance', 'browser:pure')
  }
  if (!fs.existsSync(path.join(__dirname, 'node_modules/substance-mini/dist/substance-mini.js'))){
    b.make('substance-mini')
  }
}

function _copyAssets() {
  b.copy('./node_modules/font-awesome', './build/font-awesome')
  b.copy('./fonts', './build/fonts')
  b.copy('./vendor/brace.*', './build/web/')
  b.copy('./node_modules/katex/dist/', './build/katex')
  b.copy('./node_modules/substance/dist/substance.js*', './build/web/')
  b.copy('./node_modules/substance-mini/dist/substance-mini.js*', './build/web/')
  b.copy('./node_modules/stencila-js/build/stencila-js.js*', './build/web/')
}

function _buildCss() {
  b.css('src/pagestyle/stencila.css', 'build/stencila.css', {
    variables: true
  })
}

function _buildLib(src, dest, moduleName) {
  b.js(src, {
    dest: dest,
    format: 'umd', moduleName: moduleName,
    alias: {
      'sanitize-html': path.join(__dirname, 'vendor/sanitize-html.min.js'),
    },
    external: LIB_EXTERNAL
  })
}

function _buildDocument() {
  _buildLib('src/document/document.js', 'build/stencila-document.js', 'stencilaDocument')
}

function _buildSheet() {
  _buildLib('src/sheet/sheet.js', 'build/stencila-sheet.js', 'stencilaSheet')
}

/*
  Building a single Stencila lib bundle, that stuff like Dashboard, DocumentEditor, etc.
*/
function _buildStencila() {
  _buildLib('index.es.js', 'build/stencila.js', 'stencila')
}

function _buildExamples() {
  b.copy('./examples/*/*.html', './build/')
  ;['document', 'sheet', 'dashboard'].forEach((example) => {
    b.js(`examples/${example}/app.js`, {
      dest: `build/examples/${example}/app.js`,
      format: 'umd', moduleName: `${example}Example`,
      external: EXAMPLE_EXTERNAL
    })
  })
}

b.task('clean', () => {
  b.rm('build')
})

// This is used to generate the files in ./vendor/
b.task('vendor', _buildVendor)

// ATTENTION: this is not necessary after switching to a published version of substance
b.task('deps', () => {
  _buildDeps()
})

b.task('assets', ['deps'], () => {
  _copyAssets()
})

b.task('css', ['deps'], () => {
  _buildCss()
})

b.task('document', ['assets', 'css'], () => {
  _buildDocument()
})

b.task('sheet', ['assets', 'css'], () => {
  _buildSheet()
})

b.task('examples', ['clean', 'assets', 'css', 'document', 'sheet'], () => {
  // TODO: Make all examples use the single stencila.js build, so we don't
  // need individual builds
  _buildExamples()
  _buildStencila()
})

b.task('default', ['clean', 'assets', 'examples'])

b.serve({ static: true, route: '/', folder: 'build' })
