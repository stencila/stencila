import babel from 'rollup-plugin-babel'
import nodeResolve from 'rollup-plugin-node-resolve'
import commonjs from 'rollup-plugin-commonjs'
import json from 'rollup-plugin-json'
import hypothetical from 'rollup-plugin-hypothetical'

export default {
  entry: './index.js',
  dest: './build/stencila-js.js',
  format: 'umd',
  moduleName: 'stencilaJs',
  sourceMap: true,
  plugins: [
    json({
      include: 'node_modules/**'
    }),
    hypothetical({
      allowRealFiles: true,
      files: []
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
  ],
  globals: {
  },
  acorn: {
    // Avoid error when bundling d3: "The keyword 'await' is reserved"
    // See https://github.com/rollup/rollup/issues/564#issuecomment-225302878
    allowReserved: true
  }
}
