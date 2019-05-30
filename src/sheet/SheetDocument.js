import { XMLDocument, uuid } from 'substance'
import SheetSchema from './SheetSchema'

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

  // EXPERIMENTAL
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

  getDimensions() {
    let matrix = this.getCellMatrix()
    let nrows = matrix.length
    let ncols = 0
    if (nrows > 0) {
      ncols = matrix[0].length
    }
    return [nrows, ncols]
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
