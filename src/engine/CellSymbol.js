/*
  A class representing symbols used in Stencila Cells to
  reference other cells, or ranges.

  - `x` or `doc1!x` (variable)
  - `A1` or `sheet1!A1` (cell)
  - `A1:B10` or `sheet1!A1:B10` (range)

  Symbols are only loosely bound to cells, i.e. if the graph structure
  changes, symbol-to-cell associations can change, too.

  > ATTENTION: symbols are resolved very late to allow
  for more flexibility and a better representation of broken dependencies. Therefor
  symbols must be taken literally and must carry all information to resolve the
  actual cell(s).
*/
export default class CellSymbol {

  /*
    @param {string} name of the literal
    @param {string} scope id of the owning document, such as `sheet1` or `doc1`
  */
  constructor(type, id, scope, name, mangledStr) {
    this.type = type
    this.id = id
    this.scope = scope
    this.name = name
    this.mangledStr = mangledStr
  }

  toString() {
    return this.id
  }

}
