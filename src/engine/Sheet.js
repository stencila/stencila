import { uuid, isString } from 'substance'
import { getCellLabel, getColumnLabel, qualifiedId as _qualifiedId, queryCells } from '../shared/cellHelpers'
import SheetCell from './SheetCell'
import transformRange from './transformRange'

/*
  Engine's internal model of a Spreadsheet.
*/
export default class Sheet {

  constructor(engine, data) {
    this.engine = engine
    const docId = data.id
    if (!docId) throw new Error("'id' is required")
    this.id = docId
    this.name = data.name
    // default language
    const defaultLang = data.lang || 'mini'
    this.lang = defaultLang
    if (data.hasOwnProperty('autorun')) {
      this.autorun = data.autorun
    } else {
      // TODO: using auto/ cells automatically by default
      this.autorun = true
    }
    // TODO: we can revise this as we move on
    // for now, data.cells must be present being a sequence of rows of cells.
    // data.columns is optional, but if present every data row have corresponding dimensions
    if (!data.cells) throw new Error("'cells' is mandatory")
    let ncols
    if (data.columns) {
      this.columns = data.columns
    } else {
      ncols = data.cells[0].length
      let columns = []
      for (let i = 0; i < ncols; i++) {
        columns.push({ type: 'auto' })
      }
      this.columns = columns
    }
    ncols = this.columns.length
    this.cells = data.cells.map((rowData) => {
      if (rowData.length !== ncols) throw new Error('Invalid data')
      return rowData.map(cellData => this._createCell(cellData))
    })

    if (data.onCellRegister) this.onCellRegister = data.onCellRegister
  }

  get type() { return 'sheet' }

  setAutorun(val) {
    this.autorun = val
  }

  getColumnName(colIdx) {
    let columnMeta = this.columns[colIdx]
    if (columnMeta && columnMeta.name) {
      return columnMeta.name
    } else {
      return getColumnLabel(colIdx)
    }
  }

  getCells() {
    return this.cells
  }

  queryCells(range) {
    return queryCells(this.cells, range)
  }

  updateCell(id, cellData) {
    let qualifiedId = _qualifiedId(this.id, id)
    if (isString(cellData)) {
      cellData = { source: cellData }
    }
    this.engine._updateCell(qualifiedId, cellData)
  }

  insertRows(pos, dataBlock) {
    // TODO: what if all columns and all rows had been removed
    const count = dataBlock.length
    const ncols = this.columns.length
    let block = dataBlock.map((rowData) => {
      if (rowData.length !== ncols) throw new Error('Invalid data')
      return rowData.map(cellData => this._createCell(cellData))
    })
    // transform all existing symbols and track affected cells
    let cells = this.cells
    let spans = []
    let affected = new Set()
    let visited = new Set()
    for (let i = pos; i < cells.length; i++) {
      let row = cells[i]
      for (let j = 0; j < row.length; j++) {
        let cell = row[j]
        if (cell.deps.size > 0) {
          let _spans = _transformDeps(cell, 0, pos, count, affected, visited)
          if (_spans.length > 0) {
            if (!spans[j]) spans[j] = []
            spans[j] = spans[j].concat(_spans)
          }
        }
      }
    }
    // add the spanning symbols to the deps of the new cells
    for (let i = 0; i < block.length; i++) {
      let row = block[i]
      for (let j = 0; j < row.length; j++) {
        let cell = row[j]
        if (spans[j]) cell.deps = new Set(spans[j])
      }
    }
    // update sheet structure
    this.cells.splice(pos, 0, ...block)
    this._registerCells(block)
    // TODO
    // reset state of affected cells
    // this.engine._graph.resetCells(affected)
  }

  deleteRows(pos, count) {
    let block = this.cells.slice(pos, pos+count)
    this.cells.splice(pos, count)
    this._unregisterCells(block)
  }

  insertCols(pos, dataBlock) {
    const N = this.cells.length
    if (dataBlock.length !== N) throw new Error('Invalid dimensions')
    if (dataBlock.length === 0) return
    let m = dataBlock[0].length
    let block = dataBlock.map((rowData) => {
      if (rowData.length !== m) throw new Error('Invalid data')
      return rowData.map(cellData => this._createCell(cellData))
    })
    let cols = []
    for (let i = 0; i < m; i++) {
      cols.push({ type: 'auto' })
    }
    this.columns.splice(pos, 0, ...cols)
    for (let i = 0; i < N; i++) {
      let row = this.cells[i]
      row.splice(pos, 0, ...block[i])
    }
    this._registerCells(block)
  }

  deleteCols(pos, count) {
    const N = this.cells.length
    let block = []
    this.columns.splice(pos, count)
    for (var i = 0; i < N; i++) {
      let row = this.cells[i]
      block.push(row.slice(pos, pos+count))
      row.splice(pos, count)
    }
    this._unregisterCells(block)
  }

  onCellRegister(cell) { // eslint-disable-line
  }

  _getCellSymbol(rowIdx, colIdx) {
    return `${this.id}!${getCellLabel(rowIdx, colIdx)}`
  }

  _createCell(cellData) {
    // simple format: just the expression
    if (isString(cellData)) {
      let source = cellData
      cellData = {
        id: uuid(),
        docId: this.id,
        source,
      }
    }
    let cell = new SheetCell(this, cellData)
    return cell
  }

  _registerCell(cell) {
    const engine = this.engine
    engine._registerCell(cell)
    this.onCellRegister(cell)
  }

  _unregisterCell(cell) {
    const engine = this.engine
    engine._unregisterCell(cell)
  }

  _registerCells(block) {
    if (!block) block = this.cells
    block.forEach(row => row.forEach(cell => this._registerCell(cell)))
  }

  _unregisterCells(block) {
    if (!block) block = this.cells
    block.forEach(row => row.forEach(cell => this._unregisterCell(cell)))
  }

  _removeDep(s) {
    const cells = this.cells
    for (let i = s.startRow; i <= s.endRow; i++) {
      let row = cells[i]
      for (let j = s.startCol; j <= s.endCol; j++) {
        let cell = row[j]
        cell.removeDep(s)
      }
    }
  }

  _addDep(s) {
    const cells = this.cells
    for (let i = s.startRow; i <= s.endRow; i++) {
      let row = cells[i]
      for (let j = s.startCol; j <= s.endCol; j++) {
        let cell = row[j]
        cell.addDep(s)
      }
    }
  }
}

function _transformDeps(cell, dim, pos, count, affected, visited) {
  let spans = []
  cell.deps.forEach(s => {
    if (visited.has(s)) return
    visited.add(s)
    let start, end
    if (dim === 0) {
      start = s.startRow
      end = s.endRow
    } else {
      start = s.startCol
      end = s.endCol
    }
    let res = transformRange(start, end, pos, count)
    if (!res) return
    affected.add(s.cell)
    if (res.start === start) {
      spans.push(s)
    }
    _transformSymbol(s, dim, res.start, res.end)
  })
  return spans
}

function _transformSymbol(s, dim, newStart, newEnd) {
  let origStr = s.origStr
  // transform the symbol
  // and change the source
  if (dim === 0) {
    s.startRow = newStart
    s.endRow = newEnd
  } else {
    s.startCol = newStart
    s.endCol = newEnd
  }
  let newName = getCellLabel(s.startRow, s.startCol)
  if (s.type === 'range') {
    newName += ':' + getCellLabel(s.endRow, s.endCol)
  }
  s.name = newName
  debugger

  // what to replace?
  // cell:
  //  original,
  //  transpiled,
  //  symbolMapping,
  // symbol:
  //  name
  //  origStr
  //  mangledStr

}