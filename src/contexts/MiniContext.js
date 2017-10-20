import { parse } from 'stencila-mini'
import libcore from 'stencila-libcore'

import { descendantTypes } from '../types'

export default class MiniContext {

  constructor(host) {
    this._host = host
    this._functionManager = host.functionManager
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
      return this._evaluateExpression(codeAnalysis, inputs)
    }
    return Promise.resolve(codeAnalysis)
  }

  /*
    Call a Mini function

    This gets called when evaluating a function call node within a Mini expression

  */
  callFunction(funcCall) {
    // TODO: change the signature of this by doing all mini AST related preparations before-hand
    const functionName = funcCall.name

    // Ensure the function exists
    let funcDoc = this._functionManager.getFunction(functionName)
    if (!funcDoc) {
      return _error(`Could not find function "${functionName}"`)
    }

    // Ensure there is an implementation
    let implems = funcDoc.getImplementations()
    if (implems.length === 0) {
      return _error(`Could not find implementation for function "${functionName}"`)
    }

    // TODO: Determine the best implementation language to use based on
    // where arguments reside etc
    let language = implems[0]

    // Get a context for the implementation language
    return this._host.createContext(language)
    .then((context) => {
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
      let namedArgs = funcCall.namedArgs || []
      let namedArgsMap = {}
      for (let namedArg of namedArgs) {
        let found = false
        for (let param of params) {
          if (param.name === namedArg.name) {
            found = true
            break
          }
        }
        if (!found) {
          return _error(`"${namedArg.name}" is not a valid parameter names for function "${functionName}"`)
        }
        namedArgsMap[namedArg.name] = namedArg
      }
      let argValues = []
      let index = 0
      for (let param of params) {
        const arg = args[index] || namedArgsMap[param.name]
        const value = arg ? arg.getValue() : param.default
        if (!value) {
          return _error(`Required parameter "${param.name}" was not supplied`)
        }
        if (value.type !== param.type) {
          if (descendantTypes[param.type].indexOf(value.type) < 0) {
            return _error(`Parameter "${param.name}" must be of type "${param.type}"`)
          }
        }
        argValues.push(value)
        index++
      }
      // Call the function implementation in the context, capturing any
      // messages or returning the value
      let libraryName = this._functionManager.getLibraryName(functionName)
      return context.callFunction(libraryName, functionName, argValues).then((res) => {
        if (res.messages && res.messages.length > 0) {
          funcCall.addErrors(res.messages)
          return undefined
        }
        return res.value
      })
    })

    function _error(msg) {
      funcCall.addErrors([{
        message: msg
      }])
      return new Error(msg)
    }
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

  _evaluateExpression(res, values) {
    let expr = res.expr
    return new Promise((resolve) => {
      expr.on('evaluation:finished', (val) => {
        expr.off('evaluation:finished')
        let errors = expr.root.errors
        if (errors && errors.length > 0) {
          res.messages = errors
          res.value = undefined
        } else {
          res.value = val
        }
        resolve(res)
      })
      expr.context = new ExprContext(this, values)
      expr.propagate()
    })
  }

}

/*
  This is passed as a context to a MiniExpression to resolve external symbols
  and for marshalling.
*/
class ExprContext {

  constructor(parentContext, values) {
    this.parentContext = parentContext
    this.values = values
  }

  lookup(symbol) {
    switch(symbol.type) {
      case 'var': {
        return this.values[symbol.name]
      }
      default:
        console.error('FIXME: lookup symbol', symbol)
    }
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

  callFunction(funcCall) {
    return this.parentContext.callFunction(funcCall)
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

}
