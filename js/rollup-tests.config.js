import typescript from 'rollup-plugin-typescript2'
import commonjs from 'rollup-plugin-commonjs'
import resolve from 'rollup-plugin-node-resolve'

const plugins = [
  typescript({
    tsconfigOverride: {
      compilerOptions: {
        module: 'ES2015'
      }
    },
    useTsconfigDeclarationDir: true
  }),
  resolve({
    // From https://github.com/rollup/rollup-plugin-node-resolve:
    //  some package.json files have a `browser` field which
    //  specifies alternative files to load for people bundling
    //  for the browser. If that's you, use this option, otherwise
    //  pkg.browser will be ignored
    browser: true // Default: false
  }),
  commonjs()
]

export default [/*{
  input: 'tests/unit/index.ts',
  output: {
    file: 'tests/unit/index.js',
    format: 'cjs'
  },
  plugins
},*/{
  input: 'tests/bench/browser.ts',
  output: {
    file: 'tests/bench/browser.js',
    format: 'cjs'
  },
  plugins
}]
