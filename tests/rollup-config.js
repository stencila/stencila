import babel from 'rollup-plugin-babel'
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
          let index = glob.sync('tests/*.test.js').map((f) => {
            return `import './${f}'`
          }).join('\n')
          return index
        }
      }
    },
    babel({
      // overriding babelrc
      babelrc: false,
      presets: [
        [ 'es2015', { 'modules': false } ]
      ],
      plugins: [
        'external-helpers'
      ]
    })
  ],
  // let rollup skip tape
  external: ['tape'],
  globals: {
    // instead of using tape directly
    // we want to use the one managed by the test suite
    tape: 'substanceTest.test'
  }
}
