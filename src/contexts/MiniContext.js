import { parse } from 'stencila-mini'
import { pack, unpack } from '../value'

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
    Calling into the context.

    This gets called when evaluating a Mini expression with function call statements.
  */
  callFunction(funcNode) {
    const functionName = funcNode.name
    /* new approach using FunctionManager

      - get the function document via name
      - find the best implementation (using preferred language plus args)
      - define the function in the context
      - call the function in the context

    */
    let fun = this._functionManager.getFunction(functionName)
    if (!fun) {
      let msg = `Could not resolve function "${functionName}"`
      // Note: we just return undefined and add a runtime error
      funcNode.addErrors([{
        message: msg
      }])
      return
    }
    let impls = fun.getImplementations()
    let language = impls[0]
    if (!language) {
      let msg = `Could not find implementation for function "${functionName}"`
      // Note: we just return undefined and add a runtime error
      funcNode.addErrors([{
        message: msg
      }])
      return
    }
    let libraryName = this._functionManager.getLibraryName(functionName)
    let context = this._contexts[language]

    // TODO: implement this properly
    // - choose the right context
    // - support mulitfuncs by choosing the implementation by args
    // Note: source is an expression yielding a function

    // TODO: we should get the signature here and bring the arguments into correct order
    const options = { pack: true }
    let args = []
    if (funcNode.args) {
      args = funcNode.args.map((arg) => {
        return arg.getValue()
      })
    }
    // For named arguments, just use the name and the value
    let namedArgs = {}
    if (funcNode.namedArgs) {
      for (let arg of funcNode.namedArgs) {
        namedArgs[arg.name] = arg.getValue()
      }
    }
    return _unwrapResult(
      funcNode,
      context.callFunction(libraryName, functionName, args, namedArgs, options),
      options
    )
  }

  // used to create Stencila Values
  // such as { type: 'number', data: 5 }
  marshal(type, val) {
    return {
      type, data: val
    }
  }

  plus(left, right) {
    return {
      type: 'number',
      data: left.data + right.data
    }
  }

  minus(left, right) {
    return {
      type: 'number',
      data: left.data - right.data
    }
  }

  multiply(left, right) {
    return {
      type: 'number',
      data: left.data * right.data
    }
  }

  divide(left, right) {
    return {
      type: 'number',
      data: left.data / right.data
    }
  }

  pow(left, right) {
    return {
      type: 'number',
      data: Math.pow(left.data, right.data)
    }
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
      expr, inputs, output, messages
    }
  }

  _evaluateExpression(expr) {
    return new Promise((resolve, reject) => {
      expr.on('evaluation:finished', (val) => {
        expr.off('evaluation:finished')
        resolve(val)
      })
      expr.context = this
      expr.propagate()
    })
  }
}


function _unwrapResult(funcNode, p, options) {
  return p.then((res) => {
    if (res.messages && res.messages.length > 0) {
      funcNode.addErrors(res.messages)
      return undefined
    }
    return res.value
  })
}