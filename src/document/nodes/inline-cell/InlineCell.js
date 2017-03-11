import { InlineNode } from 'substance'
import CellMixin from '../cell/CellMixin'

// TODO: Get rid of code duplication with Cell
class InlineCell extends InlineNode {
  // using an indirection for property 'expression'
  // to be able to derive extra information and parse the expression on the fly
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

}

Object.assign(InlineCell.prototype, CellMixin)

InlineCell.schema = {
  type: 'inline-cell',
  expression: { type: 'string', default: '' },
  // volatile property to store the evaluated expression
  value: { type: 'any', default: null, optional: true }
}

export default InlineCell
