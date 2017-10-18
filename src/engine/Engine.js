import { isArray, isNil, map } from 'substance'
import { parse as parseExpression } from 'stencila-mini'
import { pack, unpack, type } from '../value'
import JsContext from '../contexts/JsContext'

export default
class Engine {

  constructor(host, functionManager, options = {}) {

    this.host = host
    this.functionManager = functionManager

    // TODO: we use the cell instances to trigger events
    // alternatively we could emit events on this engine
    this._cells = {}
    this._runtimeErrors = {}

    // TODO : temporary, remove!
    this._contexts = {
      'js': new JsContext()
    }
  }

  dispose() {
    super.dispose()

    this._cells = {}
  }

  // Cell Inspection API (formerly in CellMixin)

  getExpression(cellId) {
    const cell = this.getCell(cellId)
    if (cell) {
      return cell._expr
    }
  }

  getCell(cellId) {
    return this._cells[cellId]
  }

  isPending(cellId) {
    const expr = this.getExpression(cellId)
    if (expr) {
      return expr.isPending()
    }
  }

  isReady(cellId) {
    const expr = this.getExpression(cellId)
    if (expr) {
      return expr.isReady()
    }
  }

  isDefinition(cellId) {
    const cell = this.getCell(cellId)
    return cell && cell._expr && cell._expr.isDefinition()
  }

  hasValue(cellId) {
    const expr = this.getExpression(cellId)
    return expr && !isNil(expr.getValue())
  }

  getValue(id) {
    let val = super.getValue(id)
    if (isNil(val)) {
      // TODO: in the former impl we kept the last valid value
      const expr = this.getExpression(id)
      if (expr) {
        val = expr.getValue()
      }
    }
    return val
  }

  getValueType(val) {
    return type(val)
  }

  hasErrors(cellId) {
    return this.hasRuntimeErrors(cellId) || this.hasSyntaxError(cellId)
  }

  hasRuntimeErrors(cellId) {
    if (this._runtimeErrors[cellId]) return true
    const expr = this.getExpression(cellId)
    return (expr && expr.root && expr.root.errors)
  }

  getRuntimeErrors(cellId) {
    let runtimeErrors = map(this._runtimeErrors[cellId])
    const expr = this.getExpression(cellId)
    if (expr && expr.root && expr.root.errors) {
      runtimeErrors = runtimeErrors.concat(expr.root.errors)
    }
    return runtimeErrors
  }

  // called by custom components, such as VegaLiteComponent
  addRuntimeError(cellId, key, error) {
    if (!this._runtimeErrors[cellId]) this._runtimeErrors[cellId] = {}
    if (isArray(error)) {
      error = error.map(_normalizeError)
    } else {
      error = _normalizeError(error)
    }
    this._runtimeErrors[cellId][key] = error

    function _normalizeError(error) {
      const line = error.hasOwnProperty('line') ? error.line : -1
      const column = error.hasOwnProperty('column') ? error.column : -1
      const message = error.message || error.toString()
      return {line, column, message}
    }
  }

  // called by custom components, such as VegaLiteComponent
  clearRuntimeError(cellId, key) {
    const errors = this._runtimeErrors[cellId]
    if (errors) {
      delete errors[key]
    }
  }

  hasSyntaxError(cellId) {
    const expr = this.getExpression(cellId)
    return expr && Boolean(expr.syntaxError)
  }

  getSyntaxError(cellId) {
    const expr = this.getExpression(cellId)
    if (expr) {
      return expr.syntaxError
    }
  }

  recompute(cellId) {
    // console.log('Recomputing', cellId)
    const expr = this.getExpression(cellId)
    // we can only propagate if the expression has been parsed
    // and the engine has been attached
    if (expr) {
      expr.propagate()
    }
  }

  _parse(exprStr) {
    let expr, error
    if (exprStr) {
      expr = parseExpression(exprStr)
      error = expr.syntaxError
    }
    return { expr, error }
  }

  _onEvaluationStarted(expr) {
    // console.log('evaluation started: ', expr.getSource())
    super._onEvaluationStarted(expr)
    const cell = expr._cell
    if (cell) {
      this._runtimeErrors[cell.id] = null
      cell.emit('evaluation:started', expr, cell)
    }
  }

  _onEvaluationDeferred(expr) {
    // console.log('evaluation deferred: ', expr.getSource())
    super._onEvaluationDeferred(expr)
    const cell = expr._cell
    if (cell) {
      cell.emit('evaluation:awaiting', expr, cell)
    }
  }

  _onEvaluationFinished(val, expr) {
    // console.log('evaluation finished: %s = %s', expr.getSource(), val)
    super._onEvaluationFinished(val, expr)
    const cell = expr._cell
    if (cell) {
      cell.emit('evaluation:finished', expr, cell)
    }
  }

  /*
    Calling into the context.

    There are different types of calls:
    - function calls: the arguments are positional (ATM) and passed as array
    - external cells: arguments are passed as object with
      names taken from the signature. The context is used to
      execute the sourceCode, using the arguments object.
    - chunk: like with external cells, arguments are provided
      as object. The source code is run in the same way as we know
      it from notebook cells, such as in Jupyter.
  */
  callFunction(funcNode) {
    const functionName = funcNode.name

    /* new approach using FunctionManager

      - get the function document via name
      - find the best implementation (using preferred language plus args)
      - define the function in the context
      - call the function in the context

    */

    // ATTENTION: we removed support for 'external cells' (mini + external source code)
    // as we want to evaluate how far we can get with just
    // mini + external function definitions
    // regular function calls: we need to lookup
    // TODO: find out which context to call by looking
    // up the first implementation of the function
    let fun = this.functionManager.getFunction(functionName)
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
    let libraryName = this.functionManager.getLibraryName(functionName)
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
        return pack(arg.getValue())
      })
    }
    // For named arguments, just use the name and the value
    let namedArgs = {}
    if (funcNode.namedArgs) {
      for (let arg of funcNode.namedArgs) {
        namedArgs[arg.name] = pack(arg.getValue())
      }
    }
    return _unwrapResult(
      funcNode,
      context.callFunction(libraryName, functionName, args, namedArgs, options),
      options
    )
  }

  _getContext(name) {
    // Attempt to get context from those already known to this
    // host using it's name
    return this.host.get(`name://${name}`).then(context => {
      if (context) return context
      else {
        // Determine the type of context from the context name
        let match = name.match(/([a-zA-Z]+)([\d_].+)?/)
        if (match) {
          let type = match[1]
          return this.host.post(type, name).then(context => {
            return context
          }).catch(() => {
            return null
          })
        } else {
          return null
        }
      }
    })
  }

  _lookupFunction(functionName) {
    const contexts = this._contexts
    let names = Object.keys(contexts)
    for (let i = 0; i < names.length; i++) {
      const contextName = names[i]
      const context = contexts[contextName]
      if (context.hasFunction(functionName)) {
        return { contextName, context }
      }
    }
  }

  _registerCell(cell) {
    // console.log('registering cell', cell)
    this._cells[cell.id] = cell
    if (cell.isInput()) {
      let input = cell
      const name = input.getName()
      if (name) {
        this.setValue(name, input.getValue())
      }
    } else {
      if (cell.language === 'mini') {
        let { expr, error } = this._parse(cell.source)
        if (error) {
          console.error(error)
        } else {
          cell._expr = expr
          expr._cell = cell
          this._addExpression(expr)
          this.emit('engine:updated')
        }
      } else {
        // TODO: support other languages
      }
    }
    return cell
  }

  _deregisterCell(cellId) {
    const cell = this._cells[cellId]
    if (cell) {
      if (cell.isInput()) {
        const input = cell
        const name = input.getName()
        if (name) {
          this.setValue(name, undefined)
        }
      } else {
        delete this._cells[cell.id]
        this._removeExpression(cell.id)
      }
    }
    return cell
  }

  // called by an adapter whenever the mini expression of a cell
  // has been manipulated
  _updateCell(cellId) {
    const cell = this.getCell(cellId)
    const oldExpr = this.getExpression(cellId)
    // dispose first
    if (oldExpr) {
      this._removeExpression(cellId)
    }
    if (cell.language === 'mini') {
      let { expr, error } = this._parse(cell.source)
      if (error) {
        // what to do?
      }
      if (expr) {
        this._addExpression(expr)
        cell._expr = expr
        expr._cell = cell
        // legacy: was emitted by CellMixin before
        // TODO: get rid of it if possible
        cell.emit('expression:updated', expr, cell)
      }
    }
  }

  _updateInputName(cellId, oldName) {
    const input = this.getCell(cellId)
    const newName = input.getName()
    if (oldName) {
      this.setValue(oldName, undefined)
    }
    if (newName) {
      this.setValue(newName, input.getValue(), 'propagate-immediately')
    }
  }

  _updateInputValue(cellId) {
    const input = this.getCell(cellId)
    const name = input.getName()
    if (name) {
      this.setValue(name, input.getValue(), 'propagate-immediately')
    }
  }

}

function _unwrapResult(funcNode, p, options) {
  const pack = options.pack !== false
  return p.then((res) => {
    if (res.errors) {
      funcNode.addErrors(res.errors)
      return undefined
    }
    if (res.output) {
      const output = pack ? unpack(res.output) : res.output
      return output
    }
  })
}
