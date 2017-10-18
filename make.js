/* globals __dirname, process */
const b = require('substance-bundler')
const fork = require('substance-bundler/extensions/fork')
const install = require('substance-bundler/extensions/install')
const isInstalled = require('substance-bundler/util/isInstalled')
const path = require('path')
const fs = require('fs')
const merge = require('lodash.merge')
// used to bundle example files for demo
const vfs = require('substance-bundler/extensions/vfs')

/*
  Guide: this is a bundler make file.
  It is similar to gulp in that you define tasks which
  you can connect by adding dependencies.

  This file has the following structure:
  - Constants:
    Put shared settings and literals here to avoid code duplication
  - Functions:
    All task implementations
  - Tasks:
    Task definitions

  Run `node make --help` to print usage information.
*/

// Constants
// ---------

const COMMON_SETTINGS = (custom) => { return merge({
  // paramaters that are passed to the rollup-commonjs-plugin
  commonjs: {
    namedExports: { 'acorn/dist/walk.js': [ 'simple', 'base' ] }
  }
}, custom)}

const BROWSER_EXTERNALS = {
  'substance': 'window.substance',
  'substance-texture': 'window.texture',
  'stencila-mini': 'window.stencilaMini',
  'stencila-libcore': 'window.stencilaLibCore',
  'katex': 'window.katex',
  'plotly': 'window.Plotly'
}

const EXAMPLE_EXTERNALS = Object.assign({}, BROWSER_EXTERNALS, {
  'stencila': 'window.stencila'
})

const BROWSER_TEST_EXTERNALS = Object.assign({}, BROWSER_EXTERNALS, {
  'tape': 'substanceTest.test'
})

const NODEJS_EXTERNALS = [
  'substance', 'substance-texture', 'stencila-mini', 'stencila-libcore', 'katex', 'plotly'
]

const NODEJS_TEST_EXTERNALS = NODEJS_EXTERNALS.concat(['tape', 'stream'])

// Functions
// ---------

function copyAssets() {
  b.copy('./node_modules/font-awesome', './build/font-awesome')
  b.copy('./node_modules/katex/dist/', './build/katex')
  b.copy('./node_modules/plotly.js/dist/plotly*.js*', './build/lib/')
  b.copy('./node_modules/substance/dist/substance.js*', './build/lib/')
  b.copy('./node_modules/substance-texture/dist/texture.js*', './build/lib/')
  b.copy('./node_modules/stencila-mini/dist/stencila-mini.js*', './build/lib/')
  b.copy('./node_modules/stencila-libcore/build/stencila-libcore.*', './build/lib/')
}

function buildCss() {
  b.css('src/pagestyle/stencila.css', 'build/stencila.css', {
    variables: true
  })
}

function buildStencilaBrowser() {
  b.js('index.es.js', COMMON_SETTINGS({
    dest: 'build/stencila.js',
    format: 'umd', moduleName: 'stencila',
    globals: BROWSER_EXTERNALS,
    external: NODEJS_EXTERNALS
  }))
}

function buildStencilaNodeJS() {
  b.js('index.es.js', COMMON_SETTINGS({
    dest : 'build/stencila.cjs.js',
    format: 'cjs',
    // Externals are not include into the bundle
    external: NODEJS_EXTERNALS,
  }))
}

function buildExamples() {
  b.copy('./examples/*/*.html', './build/')
  b.copy('index.html', './build/index.html')
  //
  ;['publication', 'dashboard', 'sheet'].forEach((example) => {
    b.js(`examples/${example}/app.js`, {
      dest: `build/examples/${example}/app.js`,
      format: 'umd', moduleName: `${example}Example`,
      external: EXAMPLE_EXTERNALS
    })
  })
}

// reads all test projects
function buildData() {
  // TODO: we should also be able to map images
  vfs(b, {
    src: ['./data/**/*.xml'],
    dest: 'build/vfs.js',
    format: 'umd', moduleName: 'vfs'
  })
}

// This is used to expose `STENCILA_XXXX` environment variables to the js app
function buildEnv() {
  b.custom('Creating environment variables (env.js)...', {
    dest: './build/env.js',
    execute() {
      const variables = []
      for (let name of Object.keys(process.env)) {
        if (name.match(/^STENCILA_/)) {
          variables.push(`window.${name} = "${process.env[name]}"`)
        }
      }
      b.writeSync('build/env.js', variables.join('\n'), 'utf8')
    }
  })
}

// reads all fixtures from /tests/ and writes them into a script
function buildTestBackend() {
  b.custom('Creating test backend...', {
    src: ['./tests/documents/**/*', './tests/function/fixtures/*'],
    dest: './tmp/test-vfs.js',
    execute(files) {
      const rootDir = b.rootDir
      const vfs = {}
      files.forEach((f) => {
        if (b.isDirectory(f)) return
        let content = fs.readFileSync(f).toString()
        let relPath = path.relative(rootDir, f).replace(/\\/g, '/')
        vfs[relPath] = content
      })
      const data = ['export default ', JSON.stringify(vfs, null, 2)].join('')
      b.writeSync('tmp/test-vfs.js', data)
    }
  })
}

function buildBrowserTests() {
  b.js('tests/**/*.test.js', COMMON_SETTINGS({
    dest: 'tmp/tests.js',
    format: 'umd',
    moduleName: 'tests',
    external: BROWSER_TEST_EXTERNALS
  }))
}

function buildNodeJSTests() {
  b.js('tests/**/*.test.js', COMMON_SETTINGS({
    dest: 'tmp/tests.cjs.js',
    format: 'cjs',
    external: NODEJS_TEST_EXTERNALS,
  }))
}

function buildInstrumentedTests() {
  // TODO: we must include the whole source code to see the real coverage
  // right now we only see the coverage on the files which
  // are actually imported by tests.
  b.js(['index.es.js', 'tests/**/*.test.js'], COMMON_SETTINGS({
    dest: 'tmp/tests.cov.js',
    format: 'cjs',
    istanbul: {
      include: ['src/**/*.js'],
      exclude:[]
    },
    // these should be used directly from nodejs, not bundled
    external: NODEJS_TEST_EXTERNALS.concat(['stream'])
  }))
}

function buildSingleTest(testFile) {
  const dest = path.join(__dirname, 'tmp', testFile)
  b.js(testFile, COMMON_SETTINGS({
    dest: dest,
    format: 'cjs',
    external: NODEJS_TEST_EXTERNALS
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

function buildDocumentation() {
  const config = require.resolve('./docs/docs.yml')
  fork(b, "node_modules/documentation/bin/documentation", "build", "index.es.js", "--config", config, "--output", "docs", "--format", "html")
}

function serveDocumentation() {
  const config = require.resolve('./docs/docs.yml')
  fork(b, "node_modules/documentation/bin/documentation", "serve", "--config", config, "--watch")
}

const RNG_SEARCH_DIRS = ['src/sheet', 'src/function']

function _compileSchema(name, src, options = {} ) {
  const DEST = `tmp/${name}.data.js`
  const ISSUES = `tmp/${name}.issues.txt`
  const SCHEMA = `tmp/${name}.schema.md`
  const entry = path.basename(src)
  b.custom(`Compiling schema '${name}'...`, {
    src: src,
    dest: DEST,
    execute() {
      const { compileRNG, checkSchema } = require('substance')
      const xmlSchema = compileRNG(fs, RNG_SEARCH_DIRS, entry)
      b.writeSync(DEST, `export default ${JSON.stringify(xmlSchema)}`)
      b.writeSync(SCHEMA, xmlSchema.toMD())
      if (options.debug) {
        const issues = checkSchema(xmlSchema)
        const issuesData = [`${issues.length} issues:`, ''].concat(issues).join('\n')
        b.writeSync(ISSUES, issuesData)
      }
    }
  })
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

b.task('assets', () => {
  copyAssets()
})
.describe('Copies assets into build folder.')

b.task('css', () => {
  buildCss()
})
.describe('Creates a single CSS bundle.')

b.task('schema', () => {
  _compileSchema('SheetSchema', './src/sheet/SheetSchema.rng')
  _compileSchema('FunctionSchema', './src/function/FunctionSchema.rng')
})

b.task('schema:debug', () => {
  _compileSchema('SheetSchema', './src/sheet/SheetSchema.rng', { debug: true})
  _compileSchema('FunctionSchema', './src/function/FunctionSchema.rng', { debug: true})
})

b.task('stencila', ['clean', 'assets', 'css', 'schema'], () => {
  buildData() // required by MemoryBackend
  buildEnv()
  buildStencilaBrowser()
  buildStencilaNodeJS()
})
.describe('Build the stencila library.')

b.task('examples', ['stencila'], () => {
  // TODO: Make all examples use the single stencila.js build, so we don't
  // need individual builds
  buildExamples()
})
.describe('Build the examples.')

b.task('test:backend', ['schema'], () => {
  buildTestBackend()
})

b.task('test', ['clean', 'test:backend'], () => {
  buildNodeJSTests()
  fork(b, 'node_modules/substance-test/bin/test', 'tmp/tests.cjs.js', { verbose: true })
})
.describe('Runs the tests and generates a coverage report.')

b.task('cover', ['test:backend'], () => {
  buildInstrumentedTests()
  fork(b, 'node_modules/substance-test/bin/coverage', 'tmp/tests.cov.js')
})

b.task('test:browser', ['test:backend'], () => {
  buildBrowserTests()
})

b.task('test:one', ['test:backend'], () => {
  let test = b.argv.f
  if (!test) {
    console.error("Usage: node make test:one -f <testfile>")
    process.exit(-1)
  }
  let builtTestFile = buildSingleTest(test)
  fork(b, 'node_modules/substance-test/bin/test', builtTestFile, { verbose: true })
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

b.task('default', ['stencila', 'examples'])
.describe('[stencila, examples].')

b.serve({ static: true, route: '/', folder: 'build' })
