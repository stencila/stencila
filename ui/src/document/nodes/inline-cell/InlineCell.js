import { InlineNode } from 'substance'
import CellMixin from '../cell/CellMixin'

// TODO: Get rid of code duplication with Cell
class InlineCell extends InlineNode {
  // NOTE: using indirection for all 'expression' relevant
  // properties so that we can parse the expression on-the-fly
  // and derive some extra information
  get expression() {
    return this._exprStr
  }
  set expression(expression) {
    this._setExpression(expression)
  }
  get value() {
    return this._value
  }
  set value(val) {
    this._setValue(val)
  }
  get sourceCode() {
    return this._sourceCode
  }
  set sourceCode(val) {
    this._setSourceCode(val)
  }
}

Object.assign(InlineCell.prototype, CellMixin)

InlineCell.schema = {
  type: 'inline-cell',
  expression: { type: 'string', default: '' },
  // volatile property to store the evaluated expression
  value: { type: 'any', default: null, optional: true }
}

export default InlineCell
