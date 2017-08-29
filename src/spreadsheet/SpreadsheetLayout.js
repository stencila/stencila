const DEFAULT_COLUMN_WIDTH = 100

export default class SpreadsheetLayout {

  getViewport(rowOffset, colOffset) {
    const nrows = this.getRowCount()
    const ncols = this.getColumnCount()
    const rowWindow = 20
    const columnWindow = 10
    let startRow = Math.max(0, Math.min(nrows-rowWindow-1, rowOffset))
    let startCol = Math.max(0, Math.min(ncols-columnWindow-1, colOffset))
    let endRow = Math.min(nrows-1, startRow+rowWindow)
    let endCol = Math.min(ncols-1, startCol+columnWindow)
    return { startRow, startCol, endRow, endCol }
  }

  getWidth(startCol, endCol) {
    // TODO: we should have a default width
    // plus consider columns that have an overridden custom width
    let diff = Math.abs(endCol-startCol)
    // #number of cols + row-label column width
    return (diff*DEFAULT_COLUMN_WIDTH)+50
  }

}