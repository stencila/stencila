import {parse} from 'acorn'
import {simple, base} from 'acorn/dist/walk'

import {pack, unpack} from '../packing'
import need from '../need'

let require_
if (typeof window !== 'undefined') require_ = need
else require_ = require

/**
 * A Javascript context
 *
 * Implements the Stencila `Context` API.
 */
class JsContext {

  /**
   * Run JavaScript code within the global context scope (execute a code "chunk")
   *
   * In languages like R and Python it it straight forward to assign global variables
   *
   * @param {string} code Javascript code
   * @param {object} options - Any execution options
   * @return {object} - An object with any `errors` (an object with line numbers as keys) and `outputs` (
   *                         a data package)
   *
   * @example
   *
   * // You can also assign global variables which are available in subsequent calls,
   * context.call('foo = "bar"\n\n') // { errors: {}, output: null }
   * context.call('foo') // { errors: {}, output: { type: 'str', format: 'text', value: 'bar' } }
   */
  run (code, options) {
    code = code || ''
    options = options || {}

    let error = null

    // Create a function and execute it
    try {
      (new Function('require', code))(require_) // eslint-disable-line no-new-func
    } catch (e) {
      // Catch any error
      error = e
    }

    let value
    if (!error) {
      // Evaluate the last line, and if any error then undefined result
      // This is inefficient in the sense that the last line is evaluated twice
      // but it anything else would appear to require some code parsing
      let lines = code.split('\n')
      let last = lines[lines.length - 1]
      try {
        value = (new Function('require', 'return ' + last))() // eslint-disable-line no-new-func
      } catch (error) {
        value = undefined
      }
    }

    let output
    if (value === undefined) output = null
    else output = pack(value)

    return {
      errors: this._errors(error, 0),
      output: output
    }
  }

  /**
   * Execute JavaScript code within a local function scope (execute a code "cell")
   *
   * @param {string} code - Javascript code
   * @param {object} inputs - An object with a data pack for each input variable
   * @param {object} options - Any execution options
   * @return {object} - An object with any `errors` (an object with line numbers as keys) and `outputs` (
   *                         a data package)
   *
   * @example
   *
   * // Return statement must be used to return a value
   * context.call('return 6*7') // { errors: {}, output: { type: 'int', format: 'text', value: '42' } }
   * context.call('let x = 6\nreturn x*7') // { errors: {}, output: { type: 'int', format: 'text', value: '42' } }
   *
   * // You can specify input variables (that are local to that call),
   * context.call('return Math.PI*radius', {radius:{type:'flt', format:'text', value: '21.4'}}) // { errors: {}, output: { type: 'flt', format: 'text', value: '67.23008278682157' } }
   * context.call('return radius') // { errors: { '1': 'ReferenceError: radius is not defined' }, output: null }
   */
  call (code, inputs, options) {
    code = code || ''
    inputs = inputs || {}
    options = options || {}
    if (options.pack !== false) options.pack = true

    let error = null

    // Add inputs to `locals` i.e. the execution's local scope
    let locals = {}
    for (let name in inputs) {
      locals[name] = unpack(inputs[name])
    }

    // Generate a function body. The [IIFE](https://en.wikipedia.org/wiki/Immediately-invoked_function_expression)
    // prevents errors during `buble.transform` associated with any return statements (which must be within a func)
    let body = '(function(){\n' + code + '\n})()'

    // Create a function to be executed with locals
    let func = null
    try {
      func = new Function('require', 'locals', 'with(locals){ return ' + body + '}') // eslint-disable-line no-new-func
    } catch (e) {
      // Catch a syntax error (not caught above if no transformation)
      error = e
    }

    // Execute function capturing errors and any output
    let value
    if (func) {
      try {
        value = func(require_, locals)
      } catch (e) {
        error = e
      }
    }

    let output
    if (value === undefined) output = null
    else output = options.pack ? pack(value) : value

    return {
      errors: this._errors(error, 1),
      output: output
    }
  }

  /**
   * Determine the dependencies for a piece of Javascript code
   *
   * Returns an array of all variable names not declared within
   * the piece of code. This might include global functions.
   *
   * @param  {string} code - JavaScript code
   * @return {array<string>} - A list of dependencies
   */
  depends (code) {
    let names = {}
    let ast = parse(code)
    simple(ast, {
      Identifier: node => {
        let name = node.name
        names[name] = Object.assign(names[name] || {}, {used: true})
      },
      VariableDeclarator: node => {
        let name = node.id.name
        names[name] = Object.assign(names[name] || {}, {declared: true})
      }
    }, base)
    let depends = []
    for (let name in names) {
      let usage = names[name]
      if (usage.used && !usage.declared) depends.push(name)
    }
    return depends
  }

  /**
   * Return a `null` if no error, or an object with line numbers
   * as keys and the error message as value
   *
   * @param {object} error - Error object, if any
   * @param {int} offset - Line number offset
   * @return {null|object} - A set of errors by line number
   */
  _errors (error, offset) {
    let errors = null
    if (error) {
      // Parse the error stack to get message and line number
      let lines = error.stack.split('\n')
      let message = lines[0]
      let match = lines[1].match(/<anonymous>:(\d+):\d+/)
      let line = 0
      if (match) line = parseInt(match[1]) - 2 - offset
      errors = {}
      errors[line] = message
    }
    return errors
  }

}

export default JsContext
