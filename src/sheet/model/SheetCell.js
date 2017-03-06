import { DocumentNode } from 'substance'
import '../../shared/substance/DataPatches'

import { getCellName } from './sheetHelpers'

const PREFIX_REGEXP = /^\s*([a-zA-Z0-9_@]+)?\s*=\s*/
const LITERAL = Symbol('LITERAL')
const EXPRESSION = Symbol('EXPRESSION')

export default
class SheetCell extends DocumentNode {

  // using the indirection trick to be able to derive properties
  // whenever the content is changed
  get content() {
    return this._content
  }

  set content(content) {
    this._content = content
    this._deriveProperties()
  }

  getValue() {
    return this.value
  }

  setValue(val) {
    this.value = val
    this.emit('value:updated', val)
  }

  isEmpty() {
    return !this._content
  }

  isLiteral() {
    return this._cellType === LITERAL
  }

  // only valid if this is a named expression
  getName() {
    return this._name
  }

  getExpression() {
    return this._expression
  }

  // row and col indexes are managed by Table

  getRow() {
    return this.row
  }

  getCol() {
    return this.col
  }

  getCellName() {
    return getCellName(this.getRow(), this.getCol())
  }

  _deriveProperties() {
    const content = this._content
    const prefix = PREFIX_REGEXP.exec(content)
    if (prefix) {
      this._expression = content.slice(prefix[0].length)
      const name = prefix[1]
      this._cellType = EXPRESSION
      this._name = name
    } else {
      this._cellType = LITERAL
      this._expression = content
      this.value = content
      this._name = null
    }
  }

}

SheetCell.type = "sheet-cell"

SheetCell.schema = {
  sheetId: "id",
  // plain text (aka source)
  content: { type: "string", default: "" },

  value: { type: "string", optional: true },
}
