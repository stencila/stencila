import { parse } from 'stencila-mini'
import libcore from 'stencila-libcore'

export default class MiniContext {

  // TODO: to be able to evaluate functions from mini
  // we need available contexts to dipatch to
  constructor(functionManager, contexts) {
    this._functionManager = functionManager
    this._contexts = contexts
  }

  supportsLanguage(language) {
    return Promise.resolve(language === 'mini')
  }

  analyseCode(code, exprOnly = false) {
    return Promise.resolve(this._analyseCode(code, exprOnly))
  }

  executeCode(code = '', inputs = {}, exprOnly = false) {
    let codeAnalysis = this._analyseCode(code, exprOnly)
    if (codeAnalysis.expr) {
      return this._evaluateExpression(codeAnalysis.expr)
    }
    return Promise.resolve(codeAnalysis)
  }

  /*
    Call a Mini function

    This gets called when evaluating a function call node within a Mini expression
  */
  callFunction(funcCall) {
    const functionName = funcCall.name
    /* new approach using FunctionManager

      - get the function document via name
      - find the best implementation (using preferred language plus args)
      - define the function in the context
      - call the function in the context

    */
    let funcDoc = this._functionManager.getFunction(functionName)
    if (!funcDoc) {
      let msg = `Could not resolve function "${functionName}"`
      funcCall.addErrors([{
        message: msg
      }])
      return new Error(msg)
    }
    let impls = funcDoc.getImplementations()
    let language = impls[0]
    if (!language) {
      let msg = `Could not find implementation for function "${functionName}"`
      funcCall.addErrors([{
        message: msg
      }])
      return new Error(msg)
    }
    let libraryName = this._functionManager.getLibraryName(functionName)
    let context = this._contexts[language]

    // TODO: implement this properly
    // - choose the right context

    // Generate a correctly ordered array of argument values taking into account
    // named arguments and default values
    let args = funcCall.args || []
    let namedArgs = funcCall.namedArgs || {}
    let argValues = []
    let index = 0
    for (let param of funcDoc.getParams()) {
      const arg = args[index] || namedArgs[param.name]
      const value = arg ? arg.getValue() : param.default
      if (!value) {
        const msg = 'Parameter not given and no default value available:' + param.name
        funcCall.addErrors([{
          message: msg
        }])
        return new Error(msg)
      }
      argValues.push(value)
      index++
    }

    // Cal the function implementation in the context, capturing any 
    // messages or returning the value
    return context.callFunction(libraryName, functionName, argValues).then((res) => {
      if (res.messages && res.messages.length > 0) {
        funcCall.addErrors(res.messages)
        return undefined
      }
      return res.value
    })
  }

  // used to create Stencila Values
  // such as { type: 'number', data: 5 }
  marshal(type, value) {
    // TODO: maybe there are more cases where we want to
    // cast the type according to the value
    if (type === 'number') {
      type = libcore.type(value)
    }
    return {
      type,
      data: value
    }
  }

  plus(left, right) {
    return {
      type: this._numericType(left, right),
      data: left.data + right.data
    }
  }

  minus(left, right) {
    return {
      type: this._numericType(left, right),
      data: left.data - right.data
    }
  }

  multiply(left, right) {
    return {
      type: this._numericType(left, right),
      data: left.data * right.data
    }
  }

  divide(left, right) {
    return {
      type: this._numericType(left, right),
      data: left.data / right.data
    }
  }

  pow(left, right) {
    return {
      type: this._numericType(left, right),
      data: Math.pow(left.data, right.data)
    }
  }

  _numericType(left, right) {
    let type
    if (left.type === 'integer' && right.type === 'integer') {
      type = 'integer'
    } else {
      type = 'number'
    }
    return type
  }

  _analyseCode(code) {
    let expr = parse(code)
    let inputs, output
    let messages = []
    if (expr.syntaxError) {
      messages.push({
        type: 'error',
        message: expr.syntaxError.msg
      })
    } else {
      inputs = expr.inputs.map((node)=>{
        return node.name
      })
      output = expr.name
    }
    return {
      expr,
      inputs,
      output,
      messages,
      tokens: expr.tokens
    }
  }

  _evaluateExpression(expr) {
    return new Promise((resolve) => {
      expr.on('evaluation:finished', (val) => {
        expr.off('evaluation:finished')
        resolve(val)
      })
      expr.context = this
      expr.propagate()
    })
  }

}
