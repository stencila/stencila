var b = require('substance-bundler')
var path = require('path')

// postcss extensions
var postcssScss = require('postcss-scss')
var postcssSassVariables = require('postcss-simple-vars')
var postcssNested = require('postcss-nested')
var postcssSassyImport = require('postcss-sassy-import')
var postcssSassExtend = require('postcss-sass-extend')
var postcssSassyMixins = require('postcss-sassy-mixins')
var postcssReporter = require('postcss-reporter')

function _buildVendor() {
  b.browserify('node_modules/sanitize-html/index.js', {
    dest: './vendor/sanitize-html.js',
    exports: ['default'],
    debug: false
  })
  b.minify('./vendor/sanitize-html.js', {
    debug: false
  })
  b.rm('./vendor/sanitize-html.js')
  // a custom brace bundle
  b.browserify('.make/brace.js', {
    dest: './vendor/brace.js',
    exports: ['default'],
    debug: false
  })
  b.minify('./vendor/brace.js', {
    debug: false
  })
  b.rm('./vendor/brace.js')
}

function _buildDocument(dev) {
  b.css('src/document/document.scss', 'build/document.css', {
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
    target: {
      dest: 'build/stencila-document.js',
      format: 'umd', moduleName: 'stencilaDocument'
    },
    // Ignoring stencila-js for now because
    // it needs to be re-designed to be really browser compatible
    alias: {
      'stencila-js': path.join(__dirname, 'vendor/stencila-js.stub.js'),
      'sanitize-html': path.join(__dirname, 'vendor/sanitize-html.min.js'),
      'ace': path.join(__dirname, 'vendor/brace.min.js')
    },
    commonjs: true,
    json: true
  })
}

b.task('vendor', _buildVendor)

b.task('dev:document', () => { _buildDocument('dev') })

b.task('document', () => { _buildDocument() })

b.task('dev', ['clean', 'dev:document'])

b.task('default', ['clean', 'document'])
