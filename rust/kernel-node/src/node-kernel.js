#!/usr/bin/env node

const readline = require('readline')
const vm = require('vm')

const { decodeValue, encodeValue, encodeError } = require('./node-codec')

const READY = '\u{10ACDC}\n'
const RESULT = '\u{10CB40}\n'
const TASK = '\u{10ABBA}\n'
const NEWLINE = new RegExp('\u{10B522}', 'g')

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

const LET_REGEX = /^let\s+([\w_]+)\s*=/
const CONST_REGEX = /^const\s+([\w_]+)\s*=/
const VAR_REGEX = /^var\s+([\w_]+)\s*=/
const ASSIGN_REGEX = /^\s*[\w_]+\s*=/

// Determine if a variable is defined in the context
// This needs to be done for `let` and `const` variables
// because they do not get set on the context object
function isDefined(name) {
  try {
    vm.runInContext(name, context)
  } catch (error) {
    return false
  }
  return true
}

rl.on('line', (task) => {
  const lines = task.split(NEWLINE)

  // Turn any re-declarations of variables at the top level into
  // assignments (replace with spaces to retain positions for errors and stacktraces)
  for (let index = 0; index < lines.length; index++) {
    const line = lines[index]

    const letMatch = LET_REGEX.exec(line)
    if (letMatch && isDefined(letMatch[1])) {
      lines[index] = line.replace('let', '   ')
      continue
    }

    const constMatch = CONST_REGEX.exec(line)
    if (constMatch && isDefined(constMatch[1])) {
      lines[index] = line.replace('const', '     ')
      continue
    }

    const varMatch = VAR_REGEX.exec(line)
    if (varMatch && context[varMatch[1]] !== undefined) {
      lines[index] = line.replace('var', '   ')
      continue
    }
  }

  // Ignore the output if associated with assignment on the last line
  let lastLineIsAssignment = false
  if (lines.length > 0 && ASSIGN_REGEX.test(lines[lines.length - 1])) {
    lastLineIsAssignment = true
  }

  const code = lines.join('\n')

  try {
    const output = vm.runInContext(code, context)
    if (output !== undefined && !lastLineIsAssignment) {
      const json = encodeValue(output)
      stdout.write(json + RESULT)
    }
  } catch (error) {
    const json = encodeError(error)
    stderr.write(json + RESULT)
  }
  stdout.write(TASK)
  stderr.write(TASK)
})
