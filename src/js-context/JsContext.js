import {parse} from 'acorn'
import {simple, base} from 'acorn/dist/walk'

import {pack, unpack} from '../value'
import Context from '../context/Context'

import FUNCTIONS from './functions/index'

/**
 * A Javascript context
 *
 * Implements the Stencila `Context` API. All methods return a Promise.
 *
 * @extends Context
 */
export default class JsContext extends Context {

  constructor () {
    super()
    this._functions = Object.assign({}, FUNCTIONS)
  }

  /**
   * Run JavaScript code
   *
   * @override
   */
  runCode (code, options) {
    code = code || ''
    options = options || {}
    if (options.pack !== false) options.pack = true

    let error = null

    // Create a function and execute it
    try {
      (new Function(code))() // eslint-disable-line no-new-func
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
        value = (new Function('return ' + last))() // eslint-disable-line no-new-func
      } catch (err) {
        value = undefined
      }
    }

    return Promise.resolve(
      this._result(error, 0, value, options.pack)
    )
  }

  /**
   * Call JavaScript code
   *
   * @override
   */
  callCode (code, args, options = {}) {
    const pack = (options.pack !== false)
    code = code || ''
    args = args || {}

    // Extract names and values of arguments
    let names = Object.keys(args)
    let values
    if (pack) {
      values = names.map(name => unpack(args[name]))
    } else {
      values = names.map(name => args[name])
    }

    // Execute the function with the unpacked arguments. Using `new Function` avoids call to eval
    let error = null
    let value
    try {
      const f = new Function(...names, code) // eslint-disable-line no-new-func
      value = f(...values)
    } catch (e) {
      // Catch any error
      error = e
    }

    return Promise.resolve(
      this._result(error, 0, value, pack)
    )
  }

  /**
   * Get the dependencies for a piece of code
   *
   * @override
   */
  codeDependencies (code) {
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
    for (let name in names) { // eslint-disable-line guard-for-in
      let usage = names[name]
      if (usage.used && !usage.declared) depends.push(name)
    }
    return Promise.resolve(depends)
  }


  /**
   * Does the context provide a function?
   *
   * @override
   */
  hasFunction (name) {
    return Boolean(this._functions[name])
  }

  /**
   * Call a function
   *
   * @override
   */
  callFunction (name, args, options = {}) {
    if (!name) throw new Error("'name' is mandatory")
    args = args || []
    const pack = (options.pack !== false)
    let values = args
    if (pack) {
      values = args.map(arg => unpack(arg))
    }
    let error = null
    let value
    const f = this._functions[name]
    try {
      value = f(...values)
    } catch (e) {
      error = e
    }
    return Promise.resolve(
      this._result(error, 0, value, pack)
    )
  }

  /**
   * Return a result promise
   *
   * @param {object} error - Error object, if any
   * @param {int} offset - Line number offset
   * @param {object} value - The value to be packed
   * @param {boolean} packed - Should the output be packed (or left unpacked for calls withing Javascript)
   * @return {null|object} - A set of errors by line number
   */
  _result (error, offset, value, packed) {
    if (packed !== false) packed = true

    let errors = null
    if (error) {
      // Parse the error stack to get message and line number
      let lines = error.stack.split('\n')
      let message = lines[0]
      let match = lines[1].match(/<anonymous>:(\d+):\d+/)
      let line = 0
      if (match) line = parseInt(match[1], 10) - 1 - offset
      errors = {}
      errors[line] = message
    }

    let output
    if (value === undefined) output = null
    else if (packed) output = pack(value)
    else output = value

    return {
      errors: errors,
      output: output
    }
  }

}
