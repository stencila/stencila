import { DocumentNode } from 'substance'

class Cell extends DocumentNode {

  isEmpty() {
    return !this.content
  }

  getName() {
    return this._name
  }

  get content() {
    return this._content
  }

  set content(content) {
    this._content = content
    this._updateDerivedProperties()
  }

  /**
   * Get the source prefix for this cell
   *
   * The prefix is the name and symbol part of the cell
   * source. e.g. for `answer = 42`, `answer =` is the prefix.
   * Used for providing additional info on cells in UI.
   */
  getPrefix() {
    var name = this.getName() || ''
    var kind = this.kind
    var symbol = Cell.static.kindToSymbol(kind)
    if (symbol) {
      if (name) return name + ' ' + symbol
      else return symbol
    }
    else if (name) return name
    else return ''
  }

  isConstant() {
    return [
      'exp','map','req','man','tes','vis','cil'
    ].indexOf(this.kind)<0
  }

  /**
   * Get the class name related to the display mode
   *
   * Defaults to `sm-clipped`
   */
  getDisplayClass() {
    if (this.displayMode==='exp') return 'sm-expanded'
    if (this.displayMode==='ove') return 'sm-overlay'
    if (this.displayMode==='cli') return 'sm-clipped'
    return 'sm-clipped'
  }

  // row and col indexes are managed by Table

  getRow() {
    return this.row
  }

  getCol() {
    return this.col
  }

  getCellId() {
    var Sheet = require('./Sheet')
    return Sheet.static.getCellId(this.row, this.col)
  }

  _updateDerivedProperties() {
    var content = this._content
    var match = /^\s*([a-zA-Z0-9_@]+)?\s*(=|:|\^|\||\?|~|_)/.exec(content)
    delete this._expr
    delete this._name
    if (match) {
      if (match[1]) {
        this._name = match[1]
      }

      var symbol = match[2]
      this.kind = Cell.static.symbolToKind(symbol)

      this._expression = content.slice(match[0].length)
    } else {
      // In C++ we distinguish between different types of literals e.g. 'num, 'str'
      // but for now, just use 'lit' in JS for "constant" lieteral expression cells.
      this.kind = 'lit'
      this.value = content
    }
  }

}

Cell.type = "sheet-cell"

Cell.schema = {
  // Kind of cell e.g. expression, maping, requirement, test
  kind: { type: "string", default: "" },

  // plain text (aka source)
  content: { type: "string", default: "" },

  // cell display mode
  displayMode: {type: "string", optional: true},

  // volatile data derived from table
  // ATM we need it as we set it during import
  // TODO: we should try to remove that from the schema
  row: "number",
  col: "number",

  // value is derived from the plain content by evaluating
  // it in an interpreter
  value: { type: "string", optional: true }, // evaluated value
  valueType: { type: "string", optional: true },
}

Cell.kindToSymbol = function(kind) {
  switch(kind) {
    case 'exp': return '='
    case 'req': return '^'
    case 'man': return '|'
    case 'tes': return '?'
    case 'vis': return '~'
    case 'cil': return '_'
    default: return ''
  }
}

Cell.symbolToKind = function(symbol) {
  switch(symbol) {
    case '=': return 'exp'
    case ':': return 'map'
    case '^': return 'req'
    case '|': return 'man'
    case '?': return 'tes'
    case '~': return 'vis'
    case '_': return 'cil'
    default: return ''
  }
}