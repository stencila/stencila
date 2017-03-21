import { isArray, isNil, map } from 'substance'
import { parse } from 'substance-mini'
import { type } from '../../../value'

export default {

  hasValue() {
    return !isNil(this.value)
  },

  hasRuntimeErrors() {
    return this.runtimeErrors
  },

  getRuntimeErrors() {
    return map(this.runtimeErrors)
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
    if (this._expr) {
      this._expr.propagate()
    }
  },

  _startWatching() {
    this._isWatching = true
  },

  _stopWatching() {
    this._isWatching = false
  },

  _setExpression(exprStr) {
    // dispose first
    if (this._expr) {
      this._expr.off(this)
    }
    // then renew
    this._exprStr = exprStr
    this._expr = null

    this._parse()
    if (this._deriveStateFromExpression) {
      this._deriveStateFromExpression()
    }

    if (this._isWatching) {
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
      expr.on('evaluation:finished', this._onEvaluationFinished, this)
    }
  },

  _onEvaluationStarted() {
    // console.log('Started evaluation on', this)
    this.pending = true
    this.runtimeErrors = null
    if (this._isWatching) {
      this.emit('evaluation:started')
    }
  },

  _onEvaluationFinished() {
    // console.log('Finished evaluation on', this)
    this.pending = false
    const newValue = this._expr.getValue()
    // console.log('setting value', newValue)
    this._setValue(newValue)
    if (this._isWatching) {
      this.emit('evaluation:finished')
    }
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
    if (this._isWatching && this._expr) {
      this._expr.propagate()
    }
  },


  // TODO: also make sure that call()/run() only have arguments with name (var, or named arg)
  _validateExpression(expr) {
    // check that if 'call()' or 'run()' is used
    // that there is only one of them.
    const nodes = expr.nodes
    let callCount = 0
    for (let i = 0; i < nodes.length; i++) {
      const node = expr.nodes[i]
      if (node.type === 'call' && (node.name === 'call' || node.name === 'run')) {
        callCount++
      }
    }
    if (callCount > 1) {
      throw new Error("Only one 'call()' or 'run()' allowed per expression.")
    }
  }

}