import { parse } from 'acorn'
import { simple, base } from 'acorn/dist/walk'
import { generate } from 'astring/src/astring'
import { isFunction } from 'substance'

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

    // Global variable names that should be ignored when determining code input during `analyseCode()`
    this._globals = [
      // A list of ES6 globals obtained using:
      //   const globals = require('globals')
      //   JSON.stringify(Object.keys(globals.es6))
      "Array","ArrayBuffer","Boolean","constructor","DataView","Date","decodeURI","decodeURIComponent",
      "encodeURI","encodeURIComponent","Error","escape","eval","EvalError","Float32Array","Float64Array",
      "Function","hasOwnProperty","Infinity","Int16Array","Int32Array","Int8Array","isFinite","isNaN",
      "isPrototypeOf","JSON","Map","Math","NaN","Number","Object","parseFloat","parseInt","Promise",
      "propertyIsEnumerable","Proxy","RangeError","ReferenceError","Reflect","RegExp","Set","String",
      "Symbol","SyntaxError","System","toLocaleString","toString","TypeError","Uint16Array","Uint32Array",
      "Uint8Array","Uint8ClampedArray","undefined","unescape","URIError","valueOf","WeakMap","WeakSet"
    ]

    this._libs = {
      core: libcore
    }
  }

  /**
   * Get the list of supported programming languages
   *
   * @override
   */
  supportedLanguages () {
    return Promise.resolve(
      ['js']
    )
  }

  /**
   * Analyse code and return the names of inputs, output and
   * implicitly returned value expression
   *
   * @override
   */
  _analyseCode (code, exprOnly = false, valueExpr = false) {
    let inputs = []
    let output = null
    let value = null
    let messages = []

    // Parse the code
    let ast
    try {
      ast = parse(code)
    } catch (error) {
      messages.push(this._packError(error))
    }

    if (messages.length === 0 && exprOnly) {
      // Check for single expression only
      let fail = false
      if (ast.body.length > 1) fail = true
      let first = ast.body[0]
      if (!fail && first) {
        let simpleExpr = false
        if (first.type === 'ExpressionStatement') {
          // Only allow simple expressions
          // See http://esprima.readthedocs.io/en/latest/syntax-tree-format.html#expressions-and-patterns
          // for a list of expression types
          let dissallowed = ['AssignmentExpression', 'UpdateExpression', 'AwaitExpression', 'Super']
          if (dissallowed.indexOf(first.expression.type) < 0) {
            simpleExpr = true
          }
        }
        fail = !simpleExpr
      }
      if (fail) messages.push(this._packError(new Error ('Code is not a single, simple expression')))
    }

    if (messages.length === 0) {
      // Determine which names are declared and which are used
      let declared = []
      simple(ast, {
        VariableDeclarator: node => {
          declared.push(node.id.name)
        },
        Identifier: node => {
          let name = node.name
          if (declared.indexOf(name) < 0 && this._globals.indexOf(name) < 0) inputs.push(name)
        }
      }, base)

      // If the last top level node in the AST is a VariableDeclaration or Identifier then use
      // the variable name as the output name
      let last = ast.body.pop()
      if (last) {
        if (last.type === 'VariableDeclaration') {
          output = last.declarations[0].id.name
          value = output
        } else if (last.type === 'ExpressionStatement') {
          if(last.expression.type === 'Identifier') {
            output = last.expression.name
          }
          value = generate(last)
          if (value.slice(-1) === ';') value = value.slice(0, -1)
        }
      }
    }

    let result = {
      inputs,
      output,
      messages
    }
    if (valueExpr) result.value = value
    return Promise.resolve(result)
  }

  /**
   * Execute JavaScript code
   *
   * @override
   */
  _executeCode (code = '', inputs = {}, exprOnly = false) {
    return this._analyseCode(code, exprOnly, true).then(codeAnalysis => {
      let inputNames = codeAnalysis.inputs
      let outputName = codeAnalysis.output
      let valueExpr = codeAnalysis.value
      let value
      let messages = codeAnalysis.messages
      let stdout = ''
      let stderr = ''

      let errors = messages.filter(message => message.type === 'error').length
      if (errors === 0) {
        // Extract the names and values of inputs to be used as arguments
        // (some inputs may be global and so their value in accessed directly from the function)
        let argNames = []
        let argValues = []
        inputNames.forEach(name => {
          let value = inputs[name]
          if (typeof value === 'undefined') {
            messages.push({
              line: 0,
              column: 0,
              type: 'warn',
              message: `Input variable "${name}" is not managed`
            })
          }
          else {
            argNames.push(name)
            argValues.push(this._unpackValue(value))
          }
        })

        // Capture console output functions
        let captureConsole = {
          log: function (txt) { stdout += txt },
          info: function (txt) { stdout += txt },
          warn: function (txt) { stdout += txt },
          error: function (txt) { stderr += txt }
        }
        let nullConsole = {
          log: function () {},
          info: function () {},
          warn: function () {},
          error: function () {}
        }

        // Add the return value of function to the code
        // (i.e. simulate implicit return)
        // To prevent duplication of console output
        if (valueExpr) code += `;\nconsole=nullConsole;return ${valueExpr};`

        // Execute the function with the unpacked inputs.
        try {
          const func = new Function(...argNames, 'console', 'nullConsole', code) // eslint-disable-line no-new-func
          value = func(...argValues, captureConsole, nullConsole)
        } catch (error) {
          messages.push(this._packError(error))
        }
      }

      let streams = null
      if (stdout.length || stderr.length) {
        streams = {
          stdout: stdout,
          stderr: stderr
        }
      }

      return {
        inputs: inputNames,
        output: outputName,
        value: this._packValue(value),
        messages: messages,
        streams: streams
      }
    })
  }

  libraries() {
    return Promise.resolve(this._libs)
  }

  importLibrary(library) {
    this._libs[library.name] = library
  }

  /**
   * Does the context provide a function?
   *
   * @override
   */
  hasFunction (libName, functionName) {
    let has = false
    const lib = this._libs[libName]
    if (lib) {
      if (lib[functionName]) has = true
    }
    return Promise.resolve(has)
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

    let func = lib.funcs[functionName]
    if (!func) throw new Error('No function with name: ' + functionName)

    let funcBody = func.body
    if (!isFunction(funcBody)) throw new Error(`Registered function with name ${functionName} is invalid!`)

    let values = args.map(arg => this._unpackValue(arg))

    let messages = []
    let value
    try {
      value = funcBody(...values)
    } catch (error) {
      messages.push(this._packError(error))
    }

    return Promise.resolve({
      messages: messages,
      value: this._packValue(value)
    })
  }

  /**
   * Unpack a value passed from the `Engine` or another `Context`
   */
  _unpackValue(packed) {
    return packed ? packed.data : null
  }

  /**
   * Pack a value for passing to `Engine` or another `Context`
   */
  _packValue (value) {
    if (value === undefined) return null
    let type
    if (Number.isInteger(value)) type = 'integer'
    else type = value.type || typeof value
    return {
      type: type,
      data: value
    }
  }

  /**
   * Pack an error into a {line, column, type, message} record
   *
   * @param {Error} error - Error object
   * @return {Object} - Error record
   */
  _packError (error) {
    let line = 0
    let column = 0
    let message

    if (error instanceof SyntaxError && error.loc) {
      // Get message, line and columns numbers
      line = error.loc.line
      column = error.loc.column
      message = 'SyntaxError: ' + error.message
    } else if (error.stack) {
      // Parse the error stack to get message, line and columns numbers
      let lines = error.stack.split('\n')
      let match = lines[1].match(/<anonymous>:(\d+):(\d+)/)
      if (match) {
        line = parseInt(match[1], 10) - 2
        column = parseInt(match[2], 10)
      }
      message = lines[0] || error.message
    } else {
      message = error.message
    }

    return {
      line: line,
      column: column,
      type: 'error',
      message: message
    }
  }

}
