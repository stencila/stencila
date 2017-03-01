import { Document, isNumber, forEach } from 'substance'

export default
class Sheet extends Document {

  constructor(schema) {
    super(schema)

    this._matrix = null
    this._nrows = 0
    this._ncols = 0

    this.connect(this, {
      'document:changed': this._onChange
    })
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

const ALPHABET = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"

Sheet.getColumnName = function(col) {
  if (!isNumber(col)) {
    throw new Error('Illegal argument.')
  }
  var name = ""
  while(col !== 0) {
    var mod = col % ALPHABET.length
    col = Math.floor(col/ALPHABET.length)
    name = ALPHABET[mod] + name
    if (col > 0) col--
    else if (col === 0) break
  }
  return name
}

Sheet.getColumnIndex = function(col) {
  var index = 0
  var rank = 1
  forEach(col, function(letter) {
    index += rank * ALPHABET.indexOf(letter)
    rank++
  })
  return index
}

Sheet.getCellId = function(row,col) {
  return Sheet.getColumnName(col)+(row+1)
}

Sheet.getRowCol = function(id) {
  var match = /^([A-Z]+)([1-9][0-9]*)$/.exec(id)
  return [
    parseInt(match[2], 10)-1,
    Sheet.getColumnIndex(match[1])
  ]
}
