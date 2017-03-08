import babel from 'rollup-plugin-babel'
import nodeResolve from 'rollup-plugin-node-resolve'
import commonjs from 'rollup-plugin-commonjs'
import json from 'rollup-plugin-json'
import glob from 'glob'

const START = '\0START'

export default {
  entry: START,
  dest: './build/tests.js',
  format: 'umd',
  plugins: [
    {
      resolveId (id) {
        if (id === START) return START
      },
      load (id) {
        if (id === START) {
          let index = glob.sync('tests/**/*.test.js').map((f) => {
            return `import './${f}'`
          }).join('\n')
          return index
        }
      }
    },
    json({
      include: 'node_modules/**'
    }),
    nodeResolve({
      jsnext: true
    }),
    commonjs({
      include: 'node_modules/**',
      namedExports: { 'acorn/dist/walk.js': [ 'simple', 'base' ] }
    }),
    babel({
      // overriding babelrc
      babelrc: false,
      presets: [
        [ 'es2015', { 'modules': false } ]
      ],
      plugins: [
        'external-helpers',
        ['babel-plugin-transform-builtin-extend', {
          globals: ['Error']
        }]
      ]
    })
  ],
  external: [
    // Don't bundle tape...
    'tape'
  ],
  globals: {
    // Instead, use substance-test
    tape: 'substanceTest.test'
  },
  acorn: {
    // Avoid error when bundling d3: "The keyword 'await' is reserved"
    // See https://github.com/rollup/rollup/issues/564#issuecomment-225302878
    allowReserved: true
  }
}
