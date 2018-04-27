import { uuid, isString } from 'substance'
import { getCellLabel, getColumnLabel, qualifiedId as _qualifiedId, queryCells } from '../shared/cellHelpers'
import { toIdentifier } from '../shared/expressionHelpers'
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
    if (count === 0) return
    const ncols = this.columns.length
    let block = dataBlock.map((rowData) => {
      if (rowData.length !== ncols) throw new Error('Invalid data')
      return rowData.map(cellData => this._createCell(cellData))
    })
    let affectedCells = new Set()
    let spans = _transformCells(this.engine, this.cells, 0, pos, count, affectedCells)
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
    this._sendUpdate(affectedCells)
  }

  deleteRows(pos, count) {
    if (count === 0) return
    let affectedCells = new Set()
    let block = this.cells.slice(pos, pos+count)
    _transformCells(this.engine, this.cells, 0, pos, -count, affectedCells)
    this.cells.splice(pos, count)
    this._unregisterCells(block)
    this._sendUpdate(affectedCells)
  }

  insertCols(pos, dataBlock) {
    const nrows = this.cells.length
    if (dataBlock.length !== nrows) throw new Error('Invalid dimensions')
    let count = dataBlock[0].length
    if (count === 0) return
    let affectedCells = new Set()
    // transform cells
    let spans = _transformCells(this.engine, this.cells, 1, pos, count, affectedCells)
    let block = dataBlock.map((rowData) => {
      if (rowData.length !== count) throw new Error('Invalid data')
      return rowData.map(cellData => this._createCell(cellData))
    })
    let cols = []
    for (let i = 0; i < count; i++) {
      cols.push({ type: 'auto' })
    }
    this.columns.splice(pos, 0, ...cols)
    for (let i = 0; i < nrows; i++) {
      let row = this.cells[i]
      row.splice(pos, 0, ...block[i])
    }
    // add the spanning symbols to the deps of the new cells
    for (let i = 0; i < block.length; i++) {
      let row = block[i]
      for (let j = 0; j < row.length; j++) {
        let cell = row[j]
        if (spans[i]) cell.deps = new Set(spans[i])
      }
    }
    this._registerCells(block)
    this._sendUpdate(affectedCells)
  }

  deleteCols(pos, count) {
    if (count === 0) return
    let affectedCells = new Set()
    _transformCells(this.engine, this.cells, 1, pos, -count, affectedCells)
    const N = this.cells.length
    let block = []
    this.columns.splice(pos, count)
    for (var i = 0; i < N; i++) {
      let row = this.cells[i]
      block.push(row.slice(pos, pos+count))
      row.splice(pos, count)
    }
    this._unregisterCells(block)
    this._sendUpdate(affectedCells)
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

  _sendUpdate(cells) {
    if (cells.size > 0) {
      this.engine._sendUpdate('source', cells)
    }
  }
}

function _transformCells(engine, cells, dim, pos, count, affected) {
  if (count === 0) return []
  // track updates for symbols and affected cells
  let startRow = 0
  let startCol = 0
  if (dim === 0) {
    startRow = pos
  } else {
    startCol = pos
  }
  let spans = []
  let visited = new Set()
  for (let i = startRow; i < cells.length; i++) {
    let row = cells[i]
    for (let j = startCol; j < row.length; j++) {
      let cell = row[j]
      if (cell.deps.size > 0) {
        let _spans = _recordTransformations(cell, dim, pos, count, affected, visited)
        if (_spans.length > 0) {
          if (!spans[j]) spans[j] = []
          spans[j] = spans[j].concat(_spans)
        }
      }
    }
  }
  // update the source for all cells
  affected.forEach(_transformCell)
  // reset state of affected cells
  // TODO: let this be done by CellGraph, also making sure the cell state is reset properly
  affected.forEach(cell => {
    engine._graph._structureChanged.add(cell.id)
  })
  return spans
}


function _recordTransformations(cell, dim, pos, count, affectedCells, visited) {
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
    affectedCells.add(s.cell)
    if (count > 0 && res.start === start) {
      spans.push(s)
    }
    if (dim === 0) {
      s._update = {
        startRow: res.start,
        endRow: res.end
      }
    } else {
      s._update = {
        startCol: res.start,
        endCol: res.end
      }
    }
  })
  return spans
}

function _transformCell(cell) {
  let symbols = Array.from(cell.inputs).sort((a, b) => a.startPos - b.startPos)
  let source = cell._source
  let offset = 0
  for (let i = 0; i < symbols.length; i++) {
    let s = symbols[i]
    let update = s._update
    if (!update) continue
    delete s._update
    // 1. update the symbol
    Object.assign(s, update)
    // name
    let oldName = s.name
    let newName = getCellLabel(s.startRow, s.startCol)
    if (s.type === 'range') {
      newName += ':' + getCellLabel(s.endRow, s.endCol)
    }
    // origStr
    let oldOrigStr = s.origStr
    let newOrigStr = oldOrigStr.replace(oldName, newName)
    // mangledStr
    let oldMangledName = toIdentifier(oldName)
    let newMangledName = toIdentifier(newName)
    let oldMangledStr = s.mangledStr
    let newMangledStr = oldMangledStr.replace(oldMangledName, newMangledName)
    // start- and endPos
    let newStartPos = s.startPos + offset
    let newEndPos = newStartPos + newOrigStr.length
    // 2. replace the symbol in the source and the transpiled source
    let newSource = source.original.slice(0, s.startPos) + newOrigStr + source.original.slice(s.endPos)
    let newTranspiled = source.transpiled.slice(0, s.startPos) + newMangledStr + source.transpiled.slice(s.endPos)
    // finally write the updated values
    s.name = newName
    s.id = _qualifiedId(s.docId, newName)
    s.origStr = newOrigStr
    s.mangledStr = newMangledStr
    s.startPos = newStartPos
    s.endPos = newEndPos
    source.original = newSource
    source.transpiled = newTranspiled
    source.symbolMapping[newMangledStr] = s
    delete source.symbolMapping[oldMangledStr]
    // update the offset if the source is getting longer because of this change
    // this has an effect on all subsequent symbols
    offset += newOrigStr.length - oldOrigStr.length
  }
}