import { Document, forEach } from 'substance'

export default
class SheetModel extends Document {

  constructor(schema) {
    super(schema)

    this._matrix = null
    this._nrows = 0
    this._ncols = 0

    this.on('document:changed', this._onChange, this)
  }

  getCellAt(rowIdx, colIdx) {
    if (!this._matrix) this._computeMatrix()

    var row = this._matrix[rowIdx]
    if (row) {
      var cellId = row[colIdx]
      if (cellId) {
        return this.get(cellId)
      }
    }
    return null
  }

  getRowCount() {
    if (!this._matrix) this._computeMatrix()
    return this._nrows
  }

  getColumnCount() {
    if (!this._matrix) this._computeMatrix()
    return this._ncols
  }

  _computeMatrix() {
    this._matrix = {}
    this._nrows = 0
    this._ncols = 0
    forEach(this.getNodes(), function(node) {
      if (node.type === "sheet-cell") {
        this._updateCellMatrix(node.row, node.col, node.id)
      }
    }.bind(this))
  }

  _updateCellMatrix(rowIdx, colIdx, cellId) {
    var row = this._matrix[rowIdx]
    if (!row) {
      row = {}
      this._matrix[rowIdx] = row
    }
    if (cellId) {
      row[colIdx] = cellId
    } else {
      delete row[colIdx]
    }
    this._nrows = Math.max(this._nrows, rowIdx+1)
    this._ncols = Math.max(this._ncols, colIdx+1)
  }

  // updating the matrix whenever a cell has been created or deleted
  _onChange(change) {
    if (!this._matrix) return
    forEach(change.created, function(cell) {
      this._updateCellMatrix(cell.row, cell.col, cell.id)
    }.bind(this))
    forEach(change.deleted, function(cell) {
      this._updateCellMatrix(cell.row, cell.col, null)
    }.bind(this))
  }
}
