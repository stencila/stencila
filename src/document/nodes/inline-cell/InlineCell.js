import { InlineNode, isNil } from 'substance'
import { parse } from 'substance-expression'

// TODO: Get rid of code duplication with Cell
class InlineCell extends InlineNode {

  // using an indirection for property 'expression'
  // to be able to derive extra information and parse the expression on the fly
  get expression() {
    return this._expression
  }

  set expression(expr) {
    this._expression = expr
    this._parsedExpression = null
    this._error = null
    // TODO: as an optimization we could do this only if in the
    // real document not in a buffered one (e.g. TransactionDocument or ClipboardSnippets)
    if (expr) {
      try {
        let parsedExpression = parse(expr)
        parsedExpression.id = this.id
        this._parsedExpression = parsedExpression
      } catch (error) {
        this._error = String(error)
      }
    }
  }

  getParsedExpression() {
    return this._parsedExpression
  }

  hasValue() {
    return !isNil(this.value)
  }

  getValue() {
    return this.value
  }

  setValue(val) {
    this.value = val
    this.emit('value:changed')
  }

  hasError() {
    return Boolean(this._error)
  }

  getError() {
    return this._error
  }

}

InlineCell.schema = {
  type: 'inline-cell',
  expression: { type: 'string', default: '' },
  output: { type: 'string', optional: true },
  // volatile property to store the evaluated expression
  value: { type: 'object', default: null, optional: true }
}

export default InlineCell
