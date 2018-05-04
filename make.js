/* globals __dirname, process */
const b = require('substance-bundler')
const fork = require('substance-bundler/extensions/fork')
const install = require('substance-bundler/extensions/install')
const path = require('path')
const fs = require('fs')
const merge = require('lodash.merge')
const vfs = require('substance-bundler/extensions/vfs')

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
  'rdc-js': 'window.rdc',
  'substance-texture': 'window.texture',
  'stencila-mini': 'window.stencilaMini',
  'stencila-libcore': 'window.stencilaLibcore',
  'katex': 'window.katex',
  'plotly.js': 'window.Plotly'
}

const EXAMPLE_EXTERNALS = Object.assign({}, BROWSER_EXTERNALS, {
  'stencila': 'window.stencila'
})

const BROWSER_TEST_EXTERNALS = Object.assign({}, BROWSER_EXTERNALS, {
  'tape': 'substanceTest.test'
})

const NODEJS_EXTERNALS = [
  'substance', 'substance-texture', 'stencila-mini', 'stencila-libcore', 'katex', 'plotly.js', 'rdc-js'
]

const DIST = './dist/'

const NODEJS_IGNORE = ['plotly.js']

const NODEJS_TEST_EXTERNALS = NODEJS_EXTERNALS.concat(['tape', 'stream'])

//  Server configuraion
//  -------------------

const port = 4000
b.setServerPort(port)

b.yargs.option('d', {
  type: 'string',
  alias: 'rootDir',
  describe: 'Root directory of served archives'
})
let argv = b.yargs.argv
if (argv.d) {
  const darServer = require('dar-server')
  const rootDir = argv.d
  const archiveDir = path.resolve(path.join(__dirname, rootDir))
  darServer.serve(b.server, {
    port,
    serverUrl: 'http://localhost:'+port,
    rootDir: archiveDir,
    apiUrl: '/archives'
  })
}

b.serve({ static: true, route: '/', folder: DIST })

const RNG_SEARCH_DIRS = ['src/sheet', 'src/function']

// Tasks
// -----

b.task('clean', () => {
  b.rm(DIST)
  b.rm('tmp')
})

// This is used to generate the files in ./vendor/
b.task('vendor', buildVendor)
.describe('Creates pre-bundled files in vendor/.')

b.task('assets', () => {
  copyAssets()
})
.describe('Copies assets into dist folder.')

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

b.task('prism', bundlePrism)

b.task('build:app', buildApp)

b.task('build:examples', buildExamples)

b.task('build:env', buildEnv)

b.task('build:stencila:browser', buildStencilaBrowser)

b.task('build:stencila:nodejs', buildStencilaNodeJS)

b.task('build:instrumented-tests', buildInstrumentedTests)

b.task('build', ['build:app', 'build:examples', 'build:env', 'build:stencila:browser', 'build:stencila:nodejs'])

b.task('stencila:deps', ['schema', 'prism'])

b.task('stencila', ['clean', 'assets', 'css', 'stencila:deps', 'build'])
.describe('Build the stencila library.')

b.task('examples', ['stencila'], () => {
  // TODO: Make all examples use the single stencila.js build, so we don't
  // need individual builds
  buildExamples()
})
.describe('Build the examples.')

b.task('test', ['clean', 'stencila:deps'], () => {
  buildNodeJSTests()
  fork(b, 'node_modules/substance-test/bin/test', 'tmp/tests.cjs.js', { verbose: true, await: true })
})
.describe('Runs the tests and generates a coverage report.')

b.task('cover', ['stencila:deps', 'build:instrumented-tests'], () => {
  b.rm('coverage')
  fork(b, 'node_modules/substance-test/bin/coverage', 'tmp/tests.cov.js', { await: true })
})

b.task('test:browser', ['stencila:deps'], () => {
  buildBrowserTests()
})

b.task('test:one', ['stencila:deps'], () => {
  let test = b.argv.f
  if (!test) {
    console.error("Usage: node make test:one -f <testfile>")
    process.exit(-1)
  }
  let builtTestFile = buildSingleTest(test)
  fork(b, 'node_modules/substance-test/bin/test', builtTestFile, { verbose: true, await: true })
})
.describe('Runs the tests and generates a coverage report.')

b.task('default', ['stencila'])
.describe('[stencila].')

// Helpers
// ---------

function copyAssets() {
  b.copy('./node_modules/font-awesome', DIST+'font-awesome')
  b.copy('./node_modules/katex/dist/', DIST+'katex')
  b.copy('./node_modules/plotly.js/dist/plotly*.js*', DIST+'lib/')
  b.copy('./node_modules/substance/dist/substance.js*', DIST+'lib/')
  b.copy('./node_modules/substance-texture/dist/texture.js*', DIST+'lib/')
  b.copy('./node_modules/stencila-mini/dist/stencila-mini.js*', DIST+'lib/')
  b.copy('./node_modules/stencila-libcore/builds/stencila-envcore.*', DIST+'lib/')
  b.copy('./node_modules/stencila-libcore/builds/stencila-libcore.*', DIST+'lib/')
  b.copy('./node_modules/rdc-js/dist/rdc.js*', DIST+'lib/')
}

function buildCss() {
  b.css('src/pagestyle/stencila.css', DIST+'stencila.css', {
    variables: true
  })
}

function buildStencilaBrowser() {
  b.js('index.es.js', COMMON_SETTINGS({
    dest: DIST+'stencila.js',
    format: 'umd', moduleName: 'stencila',
    globals: BROWSER_EXTERNALS,
    external: NODEJS_EXTERNALS
  }))
}

function buildStencilaNodeJS() {
  b.js('index.es.js', COMMON_SETTINGS({
    dest : DIST+'stencila.cjs.js',
    format: 'cjs',
    // Externals are not included into the bundle
    external: NODEJS_EXTERNALS,
    ignore: NODEJS_IGNORE
  }))
}

function buildApp() {
  b.copy('./app/*.html', DIST, { root: './app/'})
  // copy('./node_modules/substance/packages/** /*.css', 'dist/styles/', { root: './node_modules/substance/'})
  // copy('./node_modules/substance/packages/** /*.css', 'dist/styles/', { root: './node_modules/substance/'})

  b.js(`./app/app.js`, {
    dest: `${DIST}app.js`,
    format: 'umd', moduleName: `StencilaExample`,
    external: EXAMPLE_EXTERNALS
  })
}

function buildExamples() {
  // TODO: we should also be able to map images
  //b.custom('Converting examples from external formats', {
  //  src: ['./examples/rmarkdown/rmarkdown.Rmd'],
  //  execute(files) {
  //    fork(b, 'node_modules/.bin/stencila-convert', 'import', './examples/rmarkdown', { verbose: true })
  //  }
  //})
  b.copy('./examples', DIST+'examples')
  vfs(b, {
    src: ['./examples/**/*'],
    dest: `${DIST}vfs.js`,
    format: 'umd', moduleName: 'vfs',
    rootDir: path.join(__dirname, 'examples')
  })
}

// This is used to expose `STENCILA_XXXX` environment variables to the js app
function buildEnv() {
  b.custom('Creating environment variables (env.js)...', {
    dest: DIST+'env.js',
    execute() {
      const variables = []
      for (let name of Object.keys(process.env)) {
        if (name.match(/^STENCILA_/)) {
          variables.push(`window.${name} = "${process.env[name]}"`)
        }
      }
      b.writeFileSync(DIST+'env.js', variables.join('\n'), 'utf8')
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
    ignore: NODEJS_IGNORE
  }))
}

function buildInstrumentedTests() {
  // NOTE: we must include the whole source code to see the real coverage
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
    external: NODEJS_TEST_EXTERNALS.concat(['stream']),
    ignore: NODEJS_IGNORE
  }))
}

function buildSingleTest(testFile) {
  const dest = path.join(__dirname, 'tmp', testFile)
  b.js(testFile, COMMON_SETTINGS({
    dest: dest,
    format: 'cjs',
    external: NODEJS_TEST_EXTERNALS,
    ignore: NODEJS_IGNORE
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

function bundlePrism() {
  // Note: we stitch together a version that contains only what we need
  // and exposing it as an es6 module
  b.custom('Bundling prism...', {
    src: [
      'node_modules/prismjs/components/prism-core.js',
      'node_modules/prismjs/components/prism-clike.js',
      'node_modules/prismjs/components/prism-r.js',
      'node_modules/prismjs/components/prism-python.js',
      'node_modules/prismjs/components/prism-sql.js',
      'node_modules/prismjs/components/prism-javascript.js'
    ],
    dest: 'tmp/prism.js',
    execute(files) {
      let chunks = ['const _self = {}']
      files.forEach((file) => {
        let basename = path.basename(file)
        let content = fs.readFileSync(file, 'utf8')
        chunks.push(`/** START ${basename} **/`)
        if (basename === 'prism-core.js') {
          // cut out the core
          let start = content.indexOf('var Prism = (function(){')
          let end = content.lastIndexOf('})();')+5
          content = content.substring(start, end)
        }
        chunks.push(content)
        chunks.push(`/** END ${basename} **/`)
      })
      chunks.push('export default Prism')
      b.writeFileSync('tmp/prism.js', chunks.join('\n'))
    }
  })
}

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
      b.writeFileSync(DEST, `export default ${JSON.stringify(xmlSchema)}`)
      b.writeFileSync(SCHEMA, xmlSchema.toMD())
      if (options.debug) {
        const issues = checkSchema(xmlSchema)
        const issuesData = [`${issues.length} issues:`, ''].concat(issues).join('\n')
        b.writeFileSync(ISSUES, issuesData)
      }
    }
  })
}
