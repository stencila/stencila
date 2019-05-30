import Cell from './Cell'

export default class SheetCell extends Cell {
  constructor(doc, cellData) {
    super(doc, cellData)

    // other cells that depend on this via cell or range expression
    this.deps = new Set()
  }

  isSheetCell() { return true }

  addDep(symbol) {
    this.deps.add(symbol)
  }

  removeDep(symbol) {
    this.deps.delete(symbol)
  }
}