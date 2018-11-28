import typescript from 'rollup-plugin-typescript2'
import resolve from 'rollup-plugin-node-resolve'

const plugins = [
  resolve(),
  typescript({
    tsconfigOverride: {
      compilerOptions: {
        module: 'ES2015'
      }
    },
    useTsconfigDeclarationDir: true
  })
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
