const {pack, unpack} = require('./packing')

/**
 * A Javascript session
 *
 * Implements the Stencila `Session` API for executing chunks
 * of Javascript code
 */
class JsSession {

  execute (code, inputs) {
    code = code || ''
    inputs = inputs || {}

    // Add inputs to local scope
    let scope = {}
    for (let name in inputs) {
      scope[name] = unpack(inputs[name])
    }

    // Generate a function body
    let body = 'with(scope) {\n'
    let lines = code.split('\n')
    for (let index = 0; index < lines.length; index++) {
      if ((index === lines.length - 1) && (lines[index].trim().length > 0)) body += 'return ' + lines[index] + ';\n'
      else body += lines[index] + ';\n'
    }
    body += '}\n'

    // Create a function to be executed within that scope
    let func = null
    let error = null
    try {
      func = Function('scope', body) // eslint-disable-line no-new-func
    } catch (e) {
      // Catch and syntax error
      error = e
    }

    // Execute function capturing errors and any output
    let output = null
    if (func) {
      try {
        output = func(scope)
      } catch (e) {
        // Catch any errors
        error = e
      }
    }

    let errors = {}
    if (error) {
      // Parse the error stack to get message and line number
      let lines = error.stack.split('\n')
      let message = lines[0]
      console.log(lines[1])
      let match = lines[1].match(/<anonymous>:(\d+):\d+/)
      let line = 0
      if (match) line = parseInt(match[1]) - 3
      errors[line] = message
    }

    if (output === undefined) output = null
    else if (output) output = pack(output)

    return {
      errors: errors,
      output: output
    }
  }

}

module.exports = JsSession
