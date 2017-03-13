/* globals __dirname, process */
const b = require('substance-bundler')
const fork = require('substance-bundler/extensions/fork')
const install = require('substance-bundler/extensions/install')
const isInstalled = require('substance-bundler/util/isInstalled')
const path = require('path')
const fs = require('fs')
const merge = require('lodash.merge')

// Helpers
// -------

function copyAssets() {
  b.copy('./node_modules/font-awesome', './build/font-awesome')
  b.copy('./vendor/brace.*', './build/web/')
  b.copy('./node_modules/katex/dist/', './build/katex')
  b.copy('./node_modules/substance/dist/substance.js*', './build/web/')
  b.copy('./node_modules/substance-mini/dist/substance-mini.js*', './build/web/')
}

function buildCss() {
  b.css('src/pagestyle/stencila.css', 'build/stencila.css', {
    variables: true
  })
}

const COMMON_SETTINGS = (custom) => { return merge({
  // paramaters that are passed to the rollup-commonjs-plugin
  commonjs: {
    namedExports: { 'acorn/dist/walk.js': [ 'simple', 'base' ] }
  },
  // for libraries that we want to include into the browser bundle
  // but need to be pre-bundled (see buildVendor())
  // we register a redirect to the the pre-bundled file
  alias: {
    'sanitize-html': path.join(__dirname, 'vendor/sanitize-html.min.js'),
  }
}, custom)}

/*
  Building a single Stencila lib bundle
*/
function buildStencila() {
  const browserTarget = {
    dest: 'build/stencila.js',
    format: 'umd', moduleName: 'stencila',
    // we need to specify how the resolve external modules
    globals: {
      'substance': 'window.substance',
      'substance-mini': 'window.substanceMini',
      'brace': 'window.ace',
      'katex': 'window.katex'
    }
  }
  const nodejsTarget = {
    dest : 'build/stencila.cjs.js',
    format: 'cjs',
  }
  b.js('index.es.js', COMMON_SETTINGS({
    targets: [browserTarget, nodejsTarget],
    // Externals are not include into the bundle
    external: ['substance', 'substance-mini', 'brace', 'katex'],
  }))
}

function buildExamples() {
  const EXAMPLE_EXTERNAL = {
    'substance': 'window.substance',
    'stencila': 'window.stencila'
  }
  b.copy('./examples/*/*.html', './build/')

  ;['document', 'sheet', 'dashboard'].forEach((example) => {
    b.js(`examples/${example}/app.js`, {
      dest: `build/examples/${example}/app.js`,
      format: 'umd', moduleName: `${example}Example`,
      external: EXAMPLE_EXTERNAL
    })
  })
}

function buildTests(target) {
  if (target === 'browser') {
    b.js('tests/**/*.test.js', COMMON_SETTINGS({
      dest: 'tmp/tests.js',
      format: 'umd',
      moduleName: 'tests',
      external: {
        'substance': 'window.substance',
        'tape': 'substanceTest.test'
      }
    }))
  } else if (target === 'nodejs') {
    b.js('tests/**/*.test.js', COMMON_SETTINGS({
      dest: 'tmp/tests.cjs.js',
      format: 'cjs',
      external: ['substance', 'tape']
    }))
  } else if (target === 'cover') {
    // TODO: we must include the whole source code to see the real coverage
    // right now we only see the coverage on the files which
    // are actually imported by tests.
    b.js(['index.es.js', 'tests/**/*.test.js'], COMMON_SETTINGS({
      dest: 'tmp/tests.cov.js',
      format: 'cjs',
      istanbul: {
        include: ['src/**/*.js']
      },
      // add all modules which can be stubbed out by an empty object
      ignore: [ 'brace', 'katex' ],
      // these should be used directly from nodejs, not bundled
      external: [
        'substance', 'substance-mini', 'tape', 'stream',
      ]
    }))
  }
}

function buildSingleTest(testFile) {
  const dest = path.join(__dirname, 'tmp', testFile)
  b.js(testFile, COMMON_SETTINGS({
    dest: dest,
    format: 'cjs',
    external: ['substance', 'tape'],
    // paramaters that are passed to the rollup-commonjs-plugin
    commonjs: {
      namedExports: { 'acorn/dist/walk.js': [ 'simple', 'base' ] }
    }
  }))
  return dest
}

// this is not run all the time
// we use it to pre-bundle vendor libraries,
// to speed up bundling within this project
// and to work around problems with using rollup on these projects
function buildVendor() {
  install(b, 'browserify', '^14.1.0')
  install(b, 'uglify-js-harmony', '^2.7.5')
  minifiedVendor('./node_modules/sanitize-html/index.js', 'sanitize-html', {
    exports: ['default']
  })
  minifiedVendor('./vendor/.brace.js', 'brace')
}

function minifiedVendor(src, name, opts = {}) {
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
function buildDeps() {
  const subsDist = path.join(__dirname, 'node_modules/substance/dist')
  if (!fs.existsSync(path.join(subsDist,'substance.js')) ||
      !fs.existsSync(path.join(subsDist, 'substance.cjs.js'))) {
    b.make('substance')
  }
}

function buildDocumentation() {
  const config = require.resolve('./docs/docs.yml')
  fork(b, "node_modules/documentation/bin/documentation", "build", "--config", config, "--output", "docs", "--format", "html")
}

function serveDocumentation() {
  const config = require.resolve('./docs/docs.yml')
  fork(b, "node_modules/documentation/bin/documentation", "serve", "--config", config, "--watch")
}

// Tasks
// -----

b.task('clean', () => {
  b.rm('build')
  b.rm('tmp')
  b.rm('coverage')
})

// This is used to generate the files in ./vendor/
b.task('vendor', buildVendor)
.describe('Creates pre-bundled files in vendor/.')

// NOTE: this will not be necessary if we depend only on npm-published libraries
// At the moment, we use substance from a git branch
b.task('deps', () => {
  buildDeps()
})

b.task('assets', () => {
  copyAssets()
})
.describe('Copies assets into build folder.')

b.task('css', () => {
  buildCss()
})
.describe('Creates a single CSS bundle.')

b.task('stencila', ['clean', 'assets', 'css'], () => {
  buildStencila()
})
.describe('Build the stencila library.')

b.task('examples', ['stencila'], () => {
  // TODO: Make all examples use the single stencila.js build, so we don't
  // need individual builds
  buildExamples()
})
.describe('Build the examples.')

b.task('test', ['clean'], () => {
  buildTests('nodejs')
  fork(b, 'node_modules/substance-test/bin/test', 'tmp/tests.cjs.js')
})
.describe('Runs the tests and generates a coverage report.')

b.task('cover', ['clean'], () => {
  buildTests('cover')
  fork(b, 'node_modules/substance-test/bin/coverage', 'tmp/tests.cov.js')
})

b.task('test:browser', () => {
  buildTests('browser')
})

b.task('test:one', () => {
  let test = b.argv.f
  if (!test) {
    console.error("Usage: node make test:one -f <testfile>")
    process.exit(-1)
  }
  let builtTestFile = buildSingleTest(test)
  fork(b, 'node_modules/substance-test/bin/test', builtTestFile)
})
.describe('Runs the tests and generates a coverage report.')


b.task('docs', () => {
  if (isInstalled('documentation')) {
    buildDocumentation()
  } else {
    console.error("Please run 'npm install documentation'.\nSkipping task.")
  }
})

b.task('docs:serve', () => {
  if (isInstalled('documentation')) {
    serveDocumentation()
  } else {
    console.error("Please run 'npm install documentation'.\nSkipping task.")
  }
})

b.task('default', ['deps', 'stencila', 'examples'])
.describe('[stencila, examples].')

b.serve({ static: true, route: '/', folder: 'build' })
