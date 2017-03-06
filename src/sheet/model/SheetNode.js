import { DocumentNode } from 'substance'

export default
class SheetNode extends DocumentNode {

  constructor(...args) {
    super(...args)

    const doc = this.getDocument()
    if (doc) {
      doc.on('document:changed', this._onChange, this)
    }
  }

  dispose() {
    const doc = this.getDocument()
    if (doc) {
      doc.off(this)
    }
  }

  getCellAt(rowIdx, colIdx) {
    if (!this._matrix) this._computeMatrix()
    const row = this._matrix[rowIdx]
    if (row) {
      return row[colIdx]
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

  getMatrix() {
    if (!this._matrix) this._computeMatrix()
    return this._matrix
  }

  _computeMatrix() {
    let nrows = 0
    let ncols = 0
    let matrix = []
    const doc = this.getDocument()
    const rows = this.cells
    for (let i = 0; i < rows.length; i++) {
      const row = rows[i]
      matrix[i] = []
      if (!row) continue
      for (let j = 0; j < row.length; j++) {
        const cellId = row[j]
        if (cellId) {
          const cell = doc.get(cellId)
          _setCell(matrix, i, j, cell)
          nrows = Math.max(i, nrows)
          ncols = Math.max(i, ncols)
          // HACK: storing row and col directly on the node
          // so that we are able to access it more easily
          // however, we should probably find a better way of doing that
          // e.g. ask the sheet for cell position
          cell.row = i
          cell.col = j
        }
      }
    }
    this._matrix = matrix
    this._nrows = nrows
    this._ncols = ncols
  }

  updateCell(row, col, content) {
    const doc = this.getDocument()
    let cell = this.getCellAt(row, col)
    if (!cell && content) {
      cell = doc.create({
        type: "sheet-cell",
        sheetId: this.id,
        row: row,
        col: col,
        content: content
      })
      // TODO: with the current data model
      // and the current Operation implementation
      // this does not work in real-time collab
      // when inserting rows or columns
      if (!this.cells[row]) {
        doc.set([this.id, 'cells', row], [])
      }
      doc.set([this.id, 'cells', row, col], cell.id)
    // delete the cell and remove it from the sheet
    } else if (cell && !content) {
      doc.set([this.id, 'cells', row, col], undefined)
      doc.delete(cell.id)
    } else if (cell && content !== cell.content) {
      doc.set([cell.id, 'content'], content)
      // HACK: setting the value to make easier to detect which cell
      // has changed
      doc.set([this.id, 'cells', row, col], cell.id)
    }
  }

  // updating the matrix whenever a cell has been added or removed from
  // the cells
  _onChange(change) {
    if (change.isAffected(this.id)) {
      this._matrix = null
    }
  }
}

function _setCell(matrix, rowIdx, colIdx, val) {
  let row = matrix[rowIdx]
  if (!row) {
    row = matrix[rowIdx] = []
  }
  row[colIdx] = val
}

SheetNode.type = 'sheet'

SheetNode.schema = {
  name: 'string',
  cells: { type: ['array', 'array', 'id'], owned:true, default: [] }
}