import { BlockNode } from 'substance'
import CellMixin from './CellMixin'

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

  _deriveStateFromExpression() {
    // check if it is an external cell or a chunk
    const expr = this._expr
    this._external = false
    if (expr) {
      const nodes = expr.nodes
      // check if there is a call() or run()
      for (let i = 0; i < nodes.length; i++) {
        const node = nodes[i]
        if (node.type === 'call') {
          if (node.name === 'call' || node.name === 'run') {
            this._external = true
            break
          }
        }
      }
    }
    if (this._external) {
      if (!this.language) {
        this.language = 'js'
      }
    }
  }
}

Object.assign(Cell.prototype, CellMixin)

Cell.type = 'cell'

Cell.schema = {
  expression: { type: 'string', default: '' },
  language: { type: 'string', optional: true },
  sourceCode: { type: 'string', optional: true },
  // volatile properties for evaluated cell
  value: { type: 'any', default: null, optional: true },
}

export default Cell
