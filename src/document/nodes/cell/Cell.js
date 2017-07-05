import { BlockNode } from 'substance'
import CellMixin from './CellMixin'
import _getContextNameForExpression from '../../_getContextNameForExpression'

class Cell extends BlockNode {
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

  get context() {
    return this._context
  }

  get sourceCode() {
    return this._sourceCode
  }

  set sourceCode(val) {
    this._setSourceCode(val)
  }

  getExpressionNode() {
    return this._expr
  }

  isExternal() {
    return Boolean(this._external)
  }

  isDefinition() {
    return this._expr && this._expr.isDefinition()
  }

  isGlobal() {
    return false
  }

  _deriveStateFromExpression() {
    // check if it is an external cell or a chunk
    const expr = this._expr
    this._external = false
    this._context = null
    if (expr) {
      let context = _getContextNameForExpression(expr)
      if (context) {
        this._external = true
        this._context = context
      }
    }
  }
}

Object.assign(Cell.prototype, CellMixin)

Cell.type = 'cell'

Cell.schema = {
  expression: { type: 'string', default: '' },
  sourceCode: { type: 'string', optional: true },
  // volatile properties for evaluated cell
  value: { type: 'any', default: null, optional: true },
}

/**
 * The default context for new external cells.
 * This value is updated whenever a user changes
 * the context for a cell
 * @type {string}
 */
Cell.contextDefault = 'js'

export default Cell
