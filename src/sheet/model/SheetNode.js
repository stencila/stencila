import { DocumentNode, forEach } from 'substance'

export default
class SheetNode extends DocumentNode {

  constructor(...args) {
    super(...args)

    this._matrix = null
    this._nrows = 0
    this._ncols = 0

    this.on('document:changed', this._onChange, this)
  }

  getCellAt(rowIdx, colIdx) {
    if (!this._matrix) this._computeMatrix()

    const doc = this.getDocument()
    const row = this._matrix[rowIdx]
    if (row) {
      const cellId = row[colIdx]
      if (cellId) {
        return doc.get(cellId)
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
    const doc = this.getDocument()
    this.cells.forEach((cellId) => {
      const cell = doc.get(cellId)
      this._updateCellMatrix(cell.row, cell.col, cell.id)
    })
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
    forEach(change.created, (node) => {
      if (node.type === 'cell') {
        const cell = node
        this._updateCellMatrix(cell.row, cell.col, cell.id)
      }
    })
    forEach(change.deleted, (node) => {
      if (node.type === 'cell') {
        const cell = node
        this._updateCellMatrix(cell.row, cell.col, null)
      }
    })
  }
}

SheetNode.type = 'sheet'

SheetNode.schema = {
  name: 'string',
  cells: { type: ['array', 'id'], owned:true, default: [] }
}