import { BlockNode } from 'substance'
import CellMixin from './CellMixin'

class Cell extends BlockNode {

  // NOTE: using an indirection for property 'expression'
  // so that we can parse the expression on-the-fly
  // and derive some extra information
  get expression() {
    return this._exprStr
  }

  set expression(expression) {
    this._setExpression(expression)
  }

  // NOTE: similar we use an indirection for property 'value'
  // so that we can keep the valueType in sync
  get value() {
    return this._value
  }

  set value(val) {
    this._setValue(val)
  }
}

Object.assign(Cell.prototype, CellMixin)

Cell.schema = {
  type: 'cell',
  expression: { type: 'string', default: '' },
  language: { type: 'string', optional: true },
  sourceCode: { type: 'string', optional: true },
  // volatile properties for evaluated cell
  value: { type: 'any', default: null, optional: true },
}

export default Cell
