import babel from 'rollup-plugin-babel'
import nodeResolve from 'rollup-plugin-node-resolve'
import commonjs from 'rollup-plugin-commonjs'
import json from 'rollup-plugin-json'
import glob from 'glob'

const START = '\0START'

export default {
  entry: START,
  dest: './dist/tests.js',
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
    }),
    nodeResolve(),
    commonjs({
      include: 'node_modules/**'
    })
  ],
  // let rollup skip tape
  external: ['tape', 'd3'],
  globals: {
    // instead of using tape directly
    // we want to use the one managed by the test suite
    tape: 'substanceTest.test',
    d3: 'd3'
  }
}
