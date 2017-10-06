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

    // Global variable names that should be ignored when determining code input during `codeAnalysis()`
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
   * Analyse code and return the names of inputs, output and 
   * implicitly returned value expression
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
        let name = node.name
        if (declared.indexOf(name) < 0 && this._globals.indexOf(name) < 0) inputs.push(name)
      }
    }, base)
    // If the last top level node in the AST is a VariableDeclaration or Identifier then use
    // the variable name as the output name
    let output = null
    let value = null
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
    return Promise.resolve({
      inputs,
      output,
      value
    })
  }

  /**
   * Execute JavaScript code
   *
   * @override
   */
  executeCode (code = '', inputs = {}) {
    return this.analyseCode(code).then(codeAnalysis => {
      let inputNames = codeAnalysis.inputs
      let outputName = codeAnalysis.output
      let valueExpr = codeAnalysis.value

      let errors = []

      // Extract names and values of inputs
      let names = Object.keys(inputs)
      let values = names.map(name => this._unpackValue(inputs[name]))

      // Add return value of function
      // (i.e. simulate implicit return)
      if (valueExpr) code += `;\nreturn ${valueExpr};`

      // Execute the function with the unpacked inputs.
      let value
      try {
        const func = new Function(...names, code) // eslint-disable-line no-new-func
        value = func(...values)
      } catch (error) {
        errors.push(error)
      }

      return {
        inputs: inputNames,
        output: outputName,
        value: this._packValue(value),
        errors: this._packErrors(errors)
      }
    })
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

    let values = args.map(arg => this._unpackValue(arg))

    let errors = []
    let value
    try {
      value = func(...values)
    } catch (error) {
      errors.push(error)
    }

    return Promise.resolve({
      errors: this._packErrors(errors),
      value: this._packValue(value)
    })
  }

  /**
   * Unpack a value passed from the `Engine` or another `Context`
   */
  _unpackValue (packed) {
    let type = packed.type
    return packed.data
  }

  /**
   * Pack a value for passing to `Engine` or another `Context`
   */
  _packValue (value) {
    if (value === undefined) return null
    
    let type = libcore.type(value)
    return {
      type: type,
      data: value
    }
  }

  /**
   * Pack errors into an array of {line, column, message} records
   *
   * @param {Array<Error>} errors - Error objects
   * @return {Array<Object>} - Error records
   */
  _packErrors (errors) {
    let packed = []
    for (let error of errors) {
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

      packed.push({
        line: line,
        column: column,
        message: message
      })
    }
    return packed
  }

}
