import { getIndexesFromRange, qualifiedId } from '../shared/cellHelpers'

/*
 * A class representing symbols used in Stencila Cells to reference other cells, or ranges.
 *
 * Examples:
 *
 * - `x` or `'My Document'!x` (variable)
 * - `A1` or `'My Sheet'!A1` (cell)
 * - `A1:B10` or `'My Sheet'!A1:B10` (range)
 *
 */
export default class CellSymbol {

  /*
   * @param {string} type 'var'|'cell'|'range'
   * @param {string} docId id of the target document
   * @param {string} name name of the variable, cell, or range, e.g. 'x', 'A1', or 'A1:B10'
   * @param {string} origStr the full original symbol, as occurred in the original source
   * @param {string} mangledStr a mangled version of origStr
   * @param {number} startPos first character in the cell's source
   * @param {number} endPos last character in the cell's source
   */
  constructor(type, docId, name, origStr, mangledStr, startPos, endPos, cell) {
    this.type = type
    this.scope = docId
    this.name = name
    this.id = qualifiedId(docId, name)
    this.origStr = origStr
    this.mangledStr = mangledStr
    this.startPos = startPos
    this.endPos = endPos
    this.cell = cell

    // these are only set for 'cell' and 'range' symbols
    this.startRow = null
    this.startCol = null
    this.endRow = null
    this.endCol = null
    if (type === 'cell') {
      let { startRow, startCol } = getIndexesFromRange(name)
      this.startRow = this.endRow = startRow
      this.startCol = this.endCol = startCol
    } else if (type === 'range') {
      let [start, end] = name.split(':')
      let { startRow, startCol, endRow, endCol } = getIndexesFromRange(start, end)
      this.startRow = startRow
      this.startCol = startCol
      this.endRow = endRow
      this.endCol = endCol
    }
  }

  toString() {
    return this.id
  }
}
