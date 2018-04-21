import { XMLDocument, uuid } from 'substance'
import SheetSchema from './SheetSchema'
import { getCellLabel } from './sheetHelpers'

export default class SheetDocument extends XMLDocument {

  constructor(...args) {
    super(...args)
    this.UUID = uuid()

    // a cached random-access version of the sheet
    // this gets invalidated whenever the structure is changed (i.e. rows or cols changed)
    // TODO: we must invalidate the matrix whenever we detect structural changes
    this._matrix = null
  }

  getDocTypeParams() {
    return SheetSchema.getDocTypeParams()
  }

  getXMLSchema() {
    return SheetSchema
  }

  getRootNode() {
    return this.get('sheet')
  }

  getName() {
    return this.getRootNode().find('name').text()
  }

  // EXPERIMENTAL: introducing
  invert(change) {
    let inverted = change.invert()
    let info = inverted.info || {}
    switch(change.info.action) {
      case 'insertRows': {
        info.action = 'deleteRows'
        break
      }
      case 'deleteRows': {
        info.action = 'insertRows'
        break
      }
      case 'insertCols': {
        info.action = 'deleteCols'
        break
      }
      case 'deleteCols': {
        info.action = 'insertCols'
        break
      }
      default:
        //
    }
    inverted.info = info
    return inverted
  }

  getColumnForCell(cellId) {
    let cell = this.get(cellId)
    let row = cell.parentNode
    let colIdx = row._childNodes.indexOf(cell.id)
    return this.getColumnMeta(colIdx)
  }

  getColumnMeta(colIdx) {
    let columns = this._getColumns()
    return columns.getChildAt(colIdx)
  }

  getCell(rowIdx, colIdx) {
    const data = this._getData()
    let row = data.getChildAt(rowIdx)
    if (row) {
      let cell = row.getChildAt(colIdx)
      return cell
    }
  }

  getCellMatrix() {
    if (!this._matrix) {
      this._matrix = this._getCellMatrix()
    }
    return this._matrix
  }

  getCellLabel(cellId) {
    let cell = this.get(cellId)
    let row = cell.parentNode
    let colIdx = row._childNodes.indexOf(cell.id)
    let rowIdx = row.parentNode._childNodes.indexOf(row.id)
    let cellLabel = getCellLabel(rowIdx, colIdx)
    return cellLabel
  }

  getCellType(cell) {
    // TODO: it might be necessary to optimize this
    let row = cell.parentNode
    // TODO: this does not work with merged cells
    let colIdx = row._childNodes.indexOf(cell.id)
    let columnMeta = this.getColumnMeta(colIdx)
    return cell.attr('type') || columnMeta.attr('type') || 'any'
  }

  getColumnIndex(col) {
    let columns = this._getColumns()
    return columns._childNodes.indexOf(col.id)
  }

  getValues(startRow, startCol, endRow, endCol) {
    let vals = []
    for (let rowIdx = startRow; rowIdx <= endRow; rowIdx++) {
      let rowVals = []
      for (let colIdx = startCol; colIdx <= endCol; colIdx++) {
        let cell = this.getCell(rowIdx, colIdx)
        rowVals.push(cell.textContent)
      }
      vals.push(rowVals)
    }
    return vals
  }

  getColumnCount() {
    const nrows = this.getRowCount()
    if (nrows > 0) {
      const data = this._getData()
      let firstRow = data.getFirstChild()
      return firstRow.getChildCount()
    } else {
      return 0
    }
  }

  getRowCount() {
    const data = this._getData()
    return data.getChildCount()
  }

  getColumnWidth(colIdx) { // eslint-disable-line
    // TODO: retrieve from model
    return 100
  }

  getRowHeight(rowIdx) { // eslint-disable-line
    // TODO: retrieve from model
    return 30
  }

  ensureRowAvailable() {
    // TODO: the UI is actually not ready yet to support a delayed rendering
    // of rows.
    return Promise.resolve(true)
  }

  getState() {
    let sheet = this.getRootNode()
    if (sheet) {
      if (!sheet.state) {
        sheet.state = this.getInitialState()
      }
      return sheet.state
    }
  }

  _apply(change) {
    super._apply(change)
    // update the matrix on structural changes
    // TODO: we could be smarter by analysing the change
    switch (change.info.action) {
      case 'insertRows':
      case 'deleteRows':
      case 'insertCols':
      case 'deleteCols': {
        this._matrix = this._getCellMatrix()
        break
      }
      default:
        //
    }
  }

  _getData() {
    if (!this._dataNode) {
      this._dataNode = this.get('data')
    }
    return this._dataNode
  }

  _getColumns() {
    if (!this._columnsNode) {
      this._columnsNode = this.getRootNode().find('columns')
    }
    return this._columnsNode
  }

  _getCellMatrix() {
    const data = this._getData()
    let matrix = []
    let rows = data.getChildren()
    let nrows = rows.length
    if (nrows === 0) return matrix
    let ncols = rows[0].getChildCount()
    for (let i = 0; i < nrows; i++) {
      let row = rows[i]
      let cells = row.getChildren()
      let m = cells.length
      if (m !== ncols) {
        throw new Error(`Invalid dimension: row ${i} has ${m} cells, expected ${ncols}.`)
      }
      matrix.push(cells)
    }
    return matrix
  }
}
