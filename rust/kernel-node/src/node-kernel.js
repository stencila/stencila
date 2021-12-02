#!/usr/bin/env node

const readline = require('readline')
const vm = require('vm')

const { decodeValue, encodeValue, encodeError } = require('./node-codec')

const resSep = '\u{10ABBA}\n'
const transSep = '\u{10ACDC}\n'

const { stdin, stdout, stderr } = process

// TODO: Write as console normally does but with `resSep` and newline at end
/*
console.log = () => stdout.write()
console.debug = () => stderr.write()
console.info = () => stderr.write()
console.warn = () => stderr.write()
console.error = () => stderr.write()
*/

const rl = readline.createInterface({
  input: stdin,
  prompt: '',
  terminal: false,
})

const context = {
  console,
  decodeValue,
  encodeValue,
  encodeError,
}
vm.createContext(context)

rl.on('line', (code) => {
  const unescaped = code.replace('\\n', '\n')
  try {
    const output = vm.runInContext(unescaped, context)
    if (output !== undefined) {
      const json = encodeValue(output)
      stdout.write(resSep)
      stdout.write(json)
    }
  } catch (error) {
    const json = encodeError(error)
    stdout.write(resSep)
    stderr.write(json)
  }
  stdout.write(transSep)
  stderr.write(transSep)
})
