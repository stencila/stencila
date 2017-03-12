import { isNil } from 'substance'
import { parse } from 'substance-mini'
import { type } from '../../../value'

export default {

  hasValue() {
    return !isNil(this.value)
  },

  hasError() {
    return Boolean(this._error)
  },

  getError() {
    return this._error
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
    this.errors = []
    if (this._isWatching) {
      this._parse()
      this.emit('expression:updated', this)
    }
  },

  _parse() {
    const exprStr = this._exprStr
    if (exprStr) {
      try {
        let expr = parse(exprStr)
        this._validateExpression(expr)
        expr.id = this.id
        expr._cell = this
        this._expr = expr
        expr.on('value:updated', this._setValue, this)
      } catch (error) {
        this.errors = [String(error)]
      }
    }
  },

  _setValue(val) {
    // console.log('Setting value', this.id, val)
    if (this._value !== val || this._expr.errors.length) {
      this._value = val
      this.valueType = type(val)
      if (this._isWatching) {
        if (this._expr) {
          this.errors = this._expr.errors
        }
        this.emit('value:updated')
      }
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