#!/usr/bin/env node

const readline = require('readline')
const vm = require('vm')

const { decodeValue, encodeValue, encodeError } = require('./node-codec')

const resSep = '\u{10ABBA}\n'
const transSep = '\u{10ACDC}\n'

const { stdin, stdout, stderr } = process

console.log = function (...args) {
  for (const arg of args) {
    stdout.write(encodeValue(arg) + resSep)
  }
}

console.debug = (message) =>
  stderr.write(
    `{"type":"CodeError","errorType":"CodeDebug","errorMessage":"${message}"}${resSep}`
  )

console.info = (message) =>
  stderr.write(
    `{"type":"CodeError","errorType":"CodeInfo","errorMessage":"${message}"}${resSep}`
  )

console.warn = (message) =>
  stderr.write(
    `{"type":"CodeError","errorType":"CodeWarn","errorMessage":"${message}"}${resSep}`
  )

console.error = (message) =>
  stderr.write(
    `{"type":"CodeError","errorType":"CodeError","errorMessage":"${message}"}${resSep}`
  )

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
  const unescaped = code.replace(/\\n/g, '\n')
  try {
    const output = vm.runInContext(unescaped, context)
    if (output !== undefined) {
      const json = encodeValue(output)
      stdout.write(json + resSep)
    }
  } catch (error) {
    const json = encodeError(error)
    stderr.write(json + resSep)
  }
  stdout.write(transSep)
  stderr.write(transSep)
})
