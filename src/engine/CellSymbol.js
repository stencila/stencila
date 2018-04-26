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
   * @param {Symbol} s the parsed symbol
   * @param {string} docId id of the target document where the symbol can be resolved
   * @param {Cell} cell the owner cell which has this symbol as an input dependency
   */
  constructor({ type, scope, name, text, mangledStr, startPos, endPos}, targetDocId, cell) {
    this.type = type
    this.scope = scope
    this.name = name
    this.docId = targetDocId
    this.id = qualifiedId(targetDocId, name)
    this.origStr = text
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
