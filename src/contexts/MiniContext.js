import { parse } from 'stencila-mini'
import libcore from 'stencila-libcore'
import { getCellLabel } from '../shared/cellHelpers'
import { gather } from '../value'

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

    // Get a context for the implementation language
    let {context, library} = this._functionManager.getContextLibrary(functionName)
    // Call the function implementation in the context, capturing any
    // messages or returning the value
    let args = funcCall.args.map(arg => arg.getValue())
    let namedArgs = {}
    for (let namedArg of funcCall.namedArgs) {
      namedArgs[namedArg.name] = namedArg.getValue()
    }
    return context.callFunction(library, functionName, args, namedArgs).then((res) => {
      if (res.messages && res.messages.length > 0) {
        funcCall.addErrors(res.messages)
        return undefined
      }
      return res.value
    })

    function _error(msg) {
      console.error(msg)
      funcCall.addErrors([{
        type: 'error',
        message: msg
      }])
      return new Error(msg)
    }
  }

  _analyseCode(code) {
    if (!code) {
      return {
        inputs: [],
        output: undefined,
        messages: [],
        tokens: [],
        nodes: []
      }
    }
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
      inputs = expr.inputs.map(node => {
        // TODO: instead of interpreting the symbols
        // the mini parser should just return the symbol
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
    if (expr.syntaxError) {
      return Promise.resolve(res)
    }
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
      case 'cell': {
        // TODO: would be good to have the symbol name stored in the symbol
        let name = getCellLabel(symbol.row, symbol.col)
        return this.values[name]
      }
      case 'range': {
        // TODO: would be good to have the symbol name stored in the symbol
        let startName = getCellLabel(symbol.startRow, symbol.startCol)
        let endName = getCellLabel(symbol.endRow, symbol.endCol)
        return this.values[`${startName}_${endName}`]
      }
      default:
        throw new Error('Invalid state')
    }
  }

  // used to create Stencila Values
  // such as { type: 'number', data: 5 }
  // TODO: coerce arrays,
  marshal(type, value) {
    // TODO: maybe there are more cases where we want to
    // cast the type according to the value
    switch (type) {
      case 'number': {
        return {
          type: libcore.type(value),
          data: value
        }
      }
      case 'array': {
        return gather('array', value)
      }
      case 'range': {
        // TODO: the API is bit inconsistent here
        // range already have a correct type because
        // they are gathered by the engine
        return value
      }
      default:
        return {
          type,
          data: value
        }
    }
  }

  unmarshal(val) {
    // TODO: better understand if it is ok to make this robust
    // by guarding undefined values, and not obfuscating an error occurring elsewhere
    // it happened whenever undefined is returned by a called function
    if (!val) return undefined
    return val.data
  }

  callFunction(funcCall) {
    return this.parentContext.callFunction(funcCall)
  }

}
