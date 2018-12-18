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

export default [{
  input: 'tests/comms/webWorkerServer.ts',
  output: {
    file: 'tests/comms/webWorkerServer.js',
    format: 'cjs'
  },
  plugins
},{
  input: 'tests/comms/browserTests.ts',
  output: {
    file: 'tests/comms/browserTests.js',
    format: 'cjs'
  },
  plugins
}]
