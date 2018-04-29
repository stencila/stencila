import { uuid, isString } from 'substance'
import { qualifiedId as _qualifiedId } from '../shared/cellHelpers'
import Cell from './Cell'
import { applyCellTransformations } from './engineHelpers'

/*
  Engine's internal model of a Document.
*/
export default class Document {

  constructor(engine, data) {
    this.engine = engine
    this.data = data
    if (!data.id) throw new Error("'id' is required")
    this.id = data.id
    this.name = data.name
    this.lang = data.lang || 'mini'
    if (data.hasOwnProperty('autorun')) {
      this.autorun = data.autorun
    } else {
      // TODO: using manual execution as a default for now
      this.autorun = true
    }
    this.cells = data.cells.map(cellData => this._createCell(cellData))
    // registration hook used for propagating initial cell state to the application
    if (data.onCellRegister) this.onCellRegister = data.onCellRegister
  }

  get type() { return 'document' }

  getCells() {
    return this.cells
  }

  setAutorun(val) {
    this.autorun = val
  }

  insertCellAt(pos, cellData) {
    let cell = this._createCell(cellData)
    this._registerCell(cell)
    this.cells.splice(pos, 0, cell)
    return cell
  }

  removeCell(id) {
    const qualifiedId = _qualifiedId(this.id, id)
    const cells = this.cells
    let pos = cells.findIndex(cell => cell.id === qualifiedId)
    if (pos >= 0) {
      let cell = cells[pos]
      this.cells.splice(pos,1)
      this.engine._unregisterCell(cell)
    } else {
      console.error('Unknown cell', id)
    }
  }

  updateCell(id, cellData) {
    let qualifiedId = _qualifiedId(this.id, id)
    if (isString(cellData)) {
      cellData = { source: cellData }
    }
    this.engine._updateCell(qualifiedId, cellData)
  }

  rename(newName) {
    if (newName === this.name) return
    let graph = this.engine._graph
    let cells = this.cells
    let affectedCells = new Set()
    for (let i = 0; i < cells.length; i++) {
      let cell = cells[i]
      if (graph._cellProvidesOutput(cell)) {
        let deps = graph._ins(cell.output)
        if (deps) {
          deps.forEach(s => {
            s._update = { type: 'rename', scope: newName }
            affectedCells.add(s.cell)
          })
        }
      }
    }
    affectedCells.forEach(applyCellTransformations)
    this.name = newName
    this._sendUpdate(affectedCells)
  }

  onCellRegister(cell) { // eslint-disable-line
  }

  _createCell(cellData) {
    if (isString(cellData)) {
      let source = cellData
      cellData = {
        id: uuid(),
        docId: this.id,
        source,
        lang: this.lang
      }
    }
    return new Cell(this, cellData)
  }

  _registerCell(cell) {
    const engine = this.engine
    engine._registerCell(cell)
    this.onCellRegister(cell)
  }

  _registerCells(block) {
    if (!block) block = this.cells
    block.forEach(cell => this._registerCell(cell))
  }
}
