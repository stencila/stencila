import { EventEmitter } from 'substance'

export default class SheetViewport extends EventEmitter {

  constructor(container) {
    super()

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
    this.P = 20

  }

  getContainerWidth() {
    let el = this._container.el
    return el ? el.getWidth() : 0
  }

  getContainerHeight() {
    let el = this._container.el
    return el ? el.getHeight() : 0
  }

}