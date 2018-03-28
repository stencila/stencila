import { EventEmitter } from 'substance'
import getBoundingRect from '../util/getBoundingRect'

export default class SheetViewport extends EventEmitter {

  constructor(sheet, container) {
    super()

    this._sheet = sheet
    this._container = container

    // fictive scroll position: instead of real scroll
    // coordinates we apply a simple heuristic,
    // using a fixed height and width for every column
    // and a fictive position within this model
    this.x = 0
    this.y = 0

    // this is always the cell in the top-left corner
    this.startRow = 0
    this.startCol = 0
    // this is always the cell in the bottom-right corner
    // which is fully visible
    this.endRow = 0
    this.endCol = 0
    // size of a cell
    this.D = 30
    // number of rows to be rendered (regardless of actual container size)
    this.P = 50
  }

  getContainerWidth() {
    let el = this._container.el
    return el ? el.getWidth() : 0
  }

  getContainerHeight() {
    let el = this._container.el
    return el ? el.getHeight() : 0
  }

  getContainerRect() {
    let el = this._container.el
    return el ? getBoundingRect(el) : {}
  }

  // scrolling in a virtual grid of squares
  update(viewport) {
    let { startRow, startCol } = viewport
    let dr = startRow - this.startRow
    let dc = startCol - this.startCol
    this.startRow = startRow
    this.startCol = startCol
    this.x = startCol * this.D
    this.y = startRow * this.D
    this.emit('scroll', dr, dc)
  }

  // scrolling in a virtual grid of squares
  scroll(dx, dy) {
    const N = this.N
    const M = this.M
    let oldX = this.x
    let oldY = this.y
    let oldC = Math.floor(oldX/this.D)
    let oldR = Math.floor(oldY/this.D)
    let newX = Math.max(0, Math.min(M*this.D, oldX+dx))
    let newY = Math.max(0, Math.min(N*this.D, oldY+dy))
    this.x = newX
    this.y = newY
    let newC = Math.floor(newX/this.D)
    let newR = Math.floor(newY/this.D)
    let dr = newR - oldR
    let dc = newC - oldC
    // stop if there is no change
    if (!dr && !dc) return
    const oldStartRow = this.startRow
    const oldStartCol = this.startCol
    const newStartRow = Math.max(0, Math.min(N-1, oldStartRow+dr))
    const newStartCol = Math.max(0, Math.min(M-1, oldStartCol+dc))
    dr = newStartRow - oldStartRow
    dc = newStartCol - oldStartCol
    if (dr || dc) {
      this.startCol = newStartCol
      this.startRow = newStartRow
      this.emit('scroll', dr, dc)
    }
  }

  shift(dr, dc) {
    // just make sure that these are integers
    dr = Math.floor(dr)
    dc = Math.floor(dc)
    const sheet = this._sheet
    let M = sheet.getColumnCount()
    let N = sheet.getRowCount()
    let oldStartRow = this.startRow
    let oldStartCol = this.startCol
    let newStartRow = Math.max(0, Math.min(oldStartRow+dr, N-1))
    let newStartCol = Math.max(0, Math.min(oldStartCol+dc, M-1))
    dr = newStartRow - oldStartRow
    dc = newStartCol - oldStartCol
    if (dr || dc) {
      this.startCol = newStartCol
      this.startRow = newStartRow
      this.x = newStartCol*this.D
      this.y = newStartRow*this.D
      this.emit('scroll', dr, dc)
    }
  }

  getTotalHeight() {
    return this.N*this.D
  }

  getTotalWidth() {
    return this.M*this.D
  }

  get N() {
    return this._sheet.getRowCount()
  }

  get M() {
    return this._sheet.getColumnCount()
  }

  toJSON() {
    return {
      startRow: this.startRow,
      startCol: this.startCol
    }
  }

}
