#!/usr/bin/env node

const readline = require('readline')
const vm = require('vm')

const { decodeValue, encodeValue, encodeError } = require('./node-codec')

const READY = '\u{10ACDC}\n'
const RESULT = '\u{10CB40}\n'
const TRANS = '\u{10ABBA}\n'

const { stdin, stdout, stderr } = process

console.log = function (...args) {
  for (const arg of args) {
    stdout.write(encodeValue(arg) + RESULT)
  }
}

console.debug = (message) =>
  stderr.write(
    `{"type":"CodeError","errorType":"CodeDebug","errorMessage":"${message}"}${RESULT}`
  )

console.info = (message) =>
  stderr.write(
    `{"type":"CodeError","errorType":"CodeInfo","errorMessage":"${message}"}${RESULT}`
  )

console.warn = (message) =>
  stderr.write(
    `{"type":"CodeError","errorType":"CodeWarn","errorMessage":"${message}"}${RESULT}`
  )

console.error = (message) =>
  stderr.write(
    `{"type":"CodeError","errorType":"CodeError","errorMessage":"${message}"}${RESULT}`
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

stdout.write(READY)
stderr.write(READY)

rl.on('line', (code) => {
  const unescaped = code.replace(/\\n/g, '\n')
  try {
    const output = vm.runInContext(unescaped, context)
    if (output !== undefined) {
      const json = encodeValue(output)
      stdout.write(json + RESULT)
    }
  } catch (error) {
    const json = encodeError(error)
    stderr.write(json + RESULT)
  }
  stdout.write(TRANS)
  stderr.write(TRANS)
})
