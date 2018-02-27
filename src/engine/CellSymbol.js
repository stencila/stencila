/*
  A class representing symbols used in Stencila Cells to
  reference other cells, or ranges.

  - `x` or `doc1!x` (variable)
  - `A1` or `sheet1!A1` (cell)
  - `A1:B10` or `sheet1!A1:B10` (range)

  Symbols are only loosely bound to cells, i.e. if the graph structure
  changes, symbol-cell associations can change, too.
  In general a symbol consists of a scope and a name.
  Instead of differentiating variable, cell, and range symbols,
  we leave symbols simple, and rely on internal 'aliasing' cells for ranges.

  Sheet cells have an implicit output symbol named after their cell label,
  such as `sheet1!A1`.
  For every sheet range an internal cell is introduced, with an explicit representation
  of the dependency, such as `sheet1!A1:A2` is provided by a cell with
  `sheet1!A1` and `sheet1!A2` as input.

  > ATTENTION: symbols are resolved very late to allow
  for more flexibility and a better representation of broken dependencies. Therefor
  symbols must be taken literally and must carry all information to resolve the
  actual cell(s).

  A symbol is parsed retaining 4 different representations:

  CellSymbol.create(str, scope)


*/
export default class CellSymbol {

  /*
    @param {string} str literal as used in the cell's source
    @param {string} name of the literal
    @param {string} scope id of the owning document, such as `sheet1` or `doc1`
  */
  constructor(str, name, scope) {
    this.str = str
    this.name = name
    this.scope = scope

    this._mangledStr = _mangleStr(str)

    this._id = mangleSymbol(scope, name)
  }

  /*
    Use the fully qualified name e.g. as key for hashes.
  */
  toString() {
    return this._id
  }

  static parse(str) {
    // TODO: this is kind of redundant. If we had full client side code analysis, we could create symbols there.


  }

}

function mangleSymbol(scope, name) {
  name = _mangleStr(name)
  if (scope) {
    return `${scope}_${name}`
  } else {
    return name
  }
}

/*
  Creates a string representation that is a valid variable id in all languages (hopefully)
  The resulting string should have the same length as the original one.
*/
function _mangleStr(str) {
  return str.replace(/[!:]/g, '_')
}