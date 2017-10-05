import { parse } from 'acorn'
import { simple, base } from 'acorn/dist/walk'
import { isFunction } from 'substance'

import {pack, unpack} from '../value'
import Context from './Context'
import libcore from 'stencila-libcore'

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

    this._libs = {
      core: libcore
    }
  }

  /**
   * Get the list of supported programming language?
   *
   * @override
   */
  supportedLanguages () {
    return Promise.resolve(
      ['js']
    )
  }

  /**
   * Does the context support a programming language?
   *
   * @override
   */
  supportsLanguage (language) {
    return this.supportedLanguages().then(languages => {
      return languages.indexOf(language) > -1
    })
  }

  /**
   * Analyse code and return the names of inputs and output
   *
   * @override
   */
  analyseCode (code) {
    // Parse the code
    let ast = parse(code)
    // Determine which names are declared and which are used
    let inputs = []
    let declared = []
    simple(ast, {
      VariableDeclarator: node => {
        declared.push(node.id.name)
      },
      Identifier: node => {
        if (declared.indexOf(node.name) < 0) inputs.push(node.name)
      }
    }, base)
    // If the last top level node in the AST is a VariableDeclaration or Identifier then use
    // the variable name as the output name
    let output = null
    let last = ast.body.pop()
    if (last) {
      if (last.type === 'VariableDeclaration') {
        output = last.declarations[0].id.name
      } else if (last.type === 'ExpressionStatement') {
        if(last.expression.type === 'Identifier') {
          output = last.expression.name
        }
      }
    }
    return Promise.resolve({
      inputs,
      output
    })
  }

  /**
   * Run JavaScript code
   *
   * @override
   */
  runCode (code = '') {
    // Create a function from the code and execute it
    let error = null
    try {
      (new Function(code))() // eslint-disable-line no-new-func
    } catch (err) {
      error = err
    }

    let output
    if (!error) {
      // Evaluate the last line and if no error then make the value output
      // This is inefficient in the sense that the last line is evaluated twice
      // but alternative approaches would appear to require some code parsing
      let lines = code.split('\n')
      let last = lines[lines.length - 1]
      try {
        output = (new Function('return ' + last))() // eslint-disable-line no-new-func
      } catch (err) {
        output = undefined
      }
    }

    return Promise.resolve(
      this._result(error, output)
    )
  }

  /**
   * Call JavaScript code
   *
   * @override
   */
  callCode (code = '', inputs = {}) {
    // Extract names and values of inputs
    let names = Object.keys(inputs)
    let values = names.map(name => unpack(inputs[name]))

    // Execute the function with the unpacked inputs.
    let error = null
    let output
    try {
      const f = new Function(...names, code) // eslint-disable-line no-new-func
      output = f(...values)
    } catch (err) {
      error = err
    }

    return Promise.resolve(
      this._result(error, output)
    )
  }

  /**
   * Call a function
   *
   * @override
   */
  callFunction (libName, functionName, args = []) {
    if (!functionName) throw new Error("'name' is mandatory")

    const lib = this._libs[libName]
    if (!lib) throw new Error('No library registered with name: ' + libName)

    const func = lib[functionName]
    if (!func) throw new Error('No function with name: ' + functionName)

    if (!isFunction(func)) throw new Error(`Registered function with name ${functionName} is invalid!`)

    let values = args.map(arg => unpack(arg))

    let error = null
    let value
    try {
      value = func(...values)
    } catch (e) {
      error = e
    }

    return Promise.resolve(
      this._result(error, value)
    )
  }

  /**
   * Return a result promise
   *
   * @param {object} error - Error object, if any
   * @param {object} value - The value to be packed
   * @return {null|object} - A set of errors by line number
   */
  _result (error, value) {
    let errors = null
    if (error) {
      // Parse the error stack to get message and line number
      let lines = error.stack.split('\n')
      let match = lines[1].match(/<anonymous>:(\d+):(\d+)/)
      let line = 0
      let column = 0
      if (match) {
        line = parseInt(match[1], 10) - 1
        column = parseInt(match[2], 10)
      }
      let message = lines[0] || error.message

      errors = [{
        line: line,
        column: column,
        message: message
      }]
    }

    let output
    if (value === undefined) output = null
    else output = pack(value)

    return {
      errors: errors,
      output: output
    }
  }

}
