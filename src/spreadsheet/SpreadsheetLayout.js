const DEFAULT_COLUMN_WIDTH = 100

/*
 * Reflection of a Sheets visual dimensions.
 *
 * WIP
 */
export default class SpreadsheetLayout {

  constructor(sheet) {
    this.data = sheet.find('data')
  }

  getColumnCount() {
    if (!this._ncols) {
      this._ncols = 0
      const nrows = this.getRowCount()
      if (nrows > 0) {
        let firstRow = this.data.getFirstChild()
        this._ncols = firstRow.getChildCount()
      }
    }
    return this._ncols
  }

  getRowCount() {
    if (!this._nrows) {
      this._nrows = this.data.getChildCount()
    }
    return this._nrows
  }

  getViewport(rowOffset, colOffset) {
    const nrows = this.getRowCount()
    const ncols = this.getColumnCount()
    const rowWindow = 20
    const columnWindow = 10
    let startRow = Math.max(0, Math.min(nrows-rowWindow, rowOffset))
    let startCol = Math.max(0, Math.min(ncols-columnWindow, colOffset))
    let endRow = Math.min(nrows, startRow+rowWindow)
    let endCol = Math.min(ncols, startCol+columnWindow)
    return { startRow, startCol, endRow, endCol }
  }

  getWidth(startCol, endCol) {
    // TODO: we should have a default width
    // plus consider columns that have an overridden custom width
    let diff = Math.abs(endCol-startCol)
    // #number of cols + row-label column width
    return (diff*DEFAULT_COLUMN_WIDTH)+50
  }

  _reset() {
    this._ncols = null
    this._nrows = null
  }

}