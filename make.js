/* globals __dirname */
var b = require('substance-bundler')
var path = require('path')
var fs = require('fs')

// postcss extensions
var postcssScss = require('postcss-scss')
var postcssSassVariables = require('postcss-simple-vars')
var postcssNested = require('postcss-nested')
var postcssSassyImport = require('postcss-sassy-import')
var postcssSassExtend = require('postcss-sass-extend')
var postcssSassyMixins = require('postcss-sassy-mixins')
var postcssReporter = require('postcss-reporter')

// this is not run all the time
// we use it to pre-bundle vendor libraries,
// to speed up bundling within this project
// and to work around problems with using rollup on these projects
function _buildVendor() {
  _minifiedVendor('./node_modules/sanitize-html/index.js', 'sanitize-html', {
    exports: ['default']
  })
  _minifiedVendor('./.make/brace.js', 'brace')
  _minifiedVendor('./node_modules/emojione/lib/js/emojione.js', 'emojione', {
    standalone: true
  })
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

function _copyAssets() {
  b.copy('./node_modules/font-awesome', './build/font-awesome')
  b.copy('./vendor/brace.*', './build/web/')
  b.copy('./vendor/emojione.*', './build/web/')
  b.copy('./node_modules/emojione/assets/png', './build/web/emojione/png')
  b.copy('./node_modules/substance/dist/substance.js*', './build/web/')
}

// we need this only temporarily, or if we need to work on an
// unpublished version of substance
function _buildSubstance() {
  if (!fs.existsSync(path.join(__dirname, 'node_modules/substance/dist/substance.js'))){
    b.make('substance', 'browser:pure')
  }
}

function _buildDocument() {
  b.css('src/pagestyle/stencila.scss', 'build/stencila.css', {
    parser: postcssScss,
    // don't use predefined postcss plugins
    builtins: false,
    // ... but instead use these:
    plugins: [
      postcssSassyImport(),
      postcssSassExtend(),
      postcssSassyMixins(),
      postcssNested(),
      postcssSassVariables(),
      postcssReporter()
    ],
  })
  b.js('src/document/document.js', {
    dest: 'build/stencila-document.js',
    format: 'umd', moduleName: 'stencilaDocument',
    // Ignoring stencila-js for now because
    // it needs to be re-designed to be really browser compatible
    alias: {
      'stencila-js': path.join(__dirname, 'vendor/stencila-js.stub.js'),
      'brace': path.join(__dirname, 'vendor/brace.min.js'),
      'sanitize-html': path.join(__dirname, 'vendor/sanitize-html.min.js'),
    },
    // TODO: here we need to apply different strategies for
    // different bundles (e.g. hosted without substance, but electron one with substance)
    external: ['substance', 'emojione', 'brace'],
    commonjs: true,
    json: true
  })
}

function _buildExamples() {
  b.copy('./examples/*/*.html', './build/')
  b.js('examples/document/app.js', {
    dest: 'build/examples/document/app.js',
    format: 'umd', moduleName: 'documentExample',
    external: ['stencila-document', 'substance']
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

b.task('document', ['assets'], () => {
  _buildDocument()
})

b.task('examples', ['assets', 'document'], () => {
  _buildExamples()
})

b.task('default', ['clean', 'assets', 'document', 'examples'])

b.serve({ static: true, route: '/', folder: 'build' })
