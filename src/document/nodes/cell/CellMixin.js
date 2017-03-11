import { isNil } from 'substance'
import { parse } from 'substance-mini'
import { type } from 'stencila-js'

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

  _setExpression(exprStr) {
    // dispose first
    if (this._expr) {
      this._expr.off(this)
    }
    // then renew
    this._exprStr = exprStr
    this._expr = null
    this._error = null
    // TODO: as an optimization we could do this only if in the
    // real document not in a buffered one (e.g. TransactionDocument or ClipboardSnippets)
    if (exprStr) {
      try {
        let expr = parse(exprStr)
        this._validateExpression(expr)
        expr.id = this.id
        expr._cell = this
        this._expr = expr
        expr.on('value:updated', this._setValue, this)
      } catch (error) {
        this._error = String(error)
      }
    }
    this.emit('expression:changed', this)
  },

  _setValue(val) {
    console.log('Setting value', this.id, val)
    if (this._value !== val) {
      this._value = val
      this.valueType = type(val)
      this.emit('value:updated')
    }
  },


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