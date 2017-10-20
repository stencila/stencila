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
      return _error(`Could not resolve function "${functionName}"`)
    }
    let impls = funcDoc.getImplementations()
    let language = impls[0]
    if (!language) {
      return _error(`Could not find implementation for function "${functionName}"`)
    }
    let libraryName = this._functionManager.getLibraryName(functionName)
    let context = this._contexts[language]

    // TODO: implement this properly
    // - choose the right context

    // Generate a correctly ordered array of argument values taking into account
    // named arguments and default values and check for:
    //  - missing parameters
    //  - superfluous arguments
    //  - arguments of wrong type
    let params = funcDoc.getParams()
    let args = funcCall.args || []
    if (args.length > params.length) {
      return _error(`Too many parameters supplied (${args.length}), expected ${params.length} at most`)
    }
    let namedArgs = funcCall.namedArgs || {}
    let argValues = []
    let index = 0
    for (let param of params) {
      const arg = args[index] || namedArgs[param.name]
      const value = arg ? arg.getValue() : param.default
      if (!value) {
        return _error(`Required parameter "${param.name}" was not supplied`)
      }
      argValues.push(value)
      index++
    }
    
    function _error(msg) {
      funcCall.addErrors([{
        message: msg
      }])
      return new Error(msg)
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
    let inputs, output, tokens, nodes
    let messages = []
    if (expr.syntaxError) {
      messages.push({
        type: 'error',
        message: expr.syntaxError.msg
      })
    }
    if (expr.inputs) {
      // extract input names
      // TODO: we probably need something different, considering different
      // input types: var, cell, range
      inputs = expr.inputs.map((node)=>{
        return node.name
      })
    }
    if (expr.name) {
      output = expr.name
    }
    if (expr.tokens) {
      // some tokens are used for code highlighting
      // some for function documentation
      tokens = expr.tokens
    }

    nodes = []
    expr.nodes.forEach((n) => {
      if (n.type === 'call') {
        let args = n.args.map((a) => {
          return {
            start: a.start,
            end: a.end
          }
        }).concat(n.namedArgs.map((a) => {
          return {
            start: a.start,
            end: a.end,
            name: a.name
          }
        }))
        let node = {
          type: 'function',
          name: n.name,
          start: n.start,
          end: n.end,
          args
        }
        nodes.push(node)
      }
    })

    return {
      expr,
      inputs,
      output,
      messages,
      tokens,
      nodes
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
