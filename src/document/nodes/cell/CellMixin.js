import { isArray, isNil, map } from 'substance'
import { parse } from 'substance-mini'
import { type } from '../../../value'
import _getContextNameForExpression from '../../_getContextNameForExpression'

export default {

  isPending() {
    if (this._expr) {
      return this._expr.isPending()
    }
  },

  isReady() {
    if (this._expr) {
      return this._expr.isReady()
    }
  },

  hasValue() {
    return !isNil(this.value)
  },

  hasErrors() {
    return this.hasRuntimeErrors() || this.hasSyntaxError()
  },

  hasRuntimeErrors() {
    return this.runtimeErrors || (this._expr && this._expr.root && this._expr.root.errors)
  },

  getRuntimeErrors() {
    let runtimeErrors = map(this.runtimeErrors)
    if (this._expr && this._expr.root && this._expr.root.errors) {
      runtimeErrors = runtimeErrors.concat(this._expr.root.errors)
    }
    return runtimeErrors
  },

  addRuntimeError(key, error) {
    if (!this.runtimeErrors) this.runtimeErrors = {}
    if (isArray(error)) {
      error = error.map(_normalize)
    } else {
      error = _normalize(error)
    }
    this.runtimeErrors[key] = error

    function _normalize(error) {
      const line = error.hasOwnProperty('line') ? error.line : -1
      const column = error.hasOwnProperty('column') ? error.column : -1
      const message = error.message || error.toString()
      return {line, column, message}
    }
  },

  clearRuntimeError(key) {
    if (this.runtimeErrors) {
      delete this.runtimeErrors[key]
    }
  },

  hasSyntaxError() {
    return this._expr && Boolean(this._expr.syntaxError)
  },

  getSyntaxError() {
    return this._expr.syntaxError
  },

  recompute() {
    // we can only propagate if the expression has been parsed
    // and the engine has been attached
    if (this._expr) {
      this._expr.propagate()
    }
  },

  _isInRealDocument() {
    // NOTE: for now all cells are active which are part of a 'real' document
    return (this.document && !this.document._isTransactionDocument)
  },

  _setExpression(exprStr) {
    // dispose first
    if (this._expr) {
      this._expr.off(this)
    }
    // then renew
    this._exprStr = exprStr
    this._expr = null

    if (this._isInRealDocument()) {
      this._parse()
      this.emit('expression:updated', this)
    }
  },

  _parse() {
    const exprStr = this._exprStr
    if (exprStr) {
      let expr = parse(exprStr)
      this._validateExpression(expr)
      expr.id = this.id
      expr._cell = this
      this._expr = expr
      expr.on('evaluation:started', this._onEvaluationStarted, this)
      expr.on('evaluation:deferred', this._onEvaluationDeferred, this)
      expr.on('evaluation:finished', this._onEvaluationFinished, this)
      if (this._deriveStateFromExpression) {
        this._deriveStateFromExpression()
      }
    }
  },

  _onEvaluationStarted() {
    // console.log('Started evaluation on', this)
    this.runtimeErrors = null
    this.emit('evaluation:started')
  },

  _onEvaluationDeferred() {
    // This means that there is an evaluation coming up soon
    // it could not be done right away because a dependency is pending
    this.emit('evaluation:awaiting')
  },

  _onEvaluationFinished() {
    // console.log('Finished evaluation on', this)
    const newValue = this._expr.getValue()
    // console.log('setting value', newValue)
    this._setValue(newValue)
    this.emit('evaluation:finished')
  },

  _setValue(val) {
    // console.log('Setting value', this.id, val)
    if (this._value !== val) {
      // always keep the last computed value
      // so that UI can still render it even if
      if (!isNil(this._value)) {
        this._lastValidValue = this._value
        this._lastValidValueType = this.valueType
      }
      this._value = val
      this.valueType = type(val)
    }
  },

  _setSourceCode(val) {
    // console.log('Setting sourceCode', this.id, val)
    this._sourceCode = val
    this.recompute()
  },

  // TODO: also make sure that call()/run() only have arguments with name (var, or named arg)
  _validateExpression(expr) {
    let context = _getContextNameForExpression(expr)
    if (context) {
      if ( (expr.isDefinition() && !expr.root.rhs.type === 'call')
        || !expr.root.type === 'call') {
        throw new Error('Incorrect syntax for an external cell.')
      }
    }
  }

}
