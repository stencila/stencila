import { Expression, Engine, parse } from 'substance-mini'

export default
class SheetEngine extends Engine {

  constructor(editorSession, sheet) {
    super()

    this.editorSession = editorSession
    this.sheet = sheet

    // matrix to store cells and values
    this._data = []
    this._values['$data'] = this._data

    this._initialize()

    this.editorSession.on('render', this.onSheetUpdated, this, {
      resource: 'document',
      path: [sheet.id]
    })
  }

  dispose() {
    this.sheet.off(this)
  }

  onSheetUpdated(change) {
    const doc = this.editorSession.getDocument()
    const sheet = this.sheet
    // HACK: this is heavily exploiting assumptions about
    // the how the sheet gets updated (see SheetNode.updateCell())
    for (let i = 0; i < change.ops.length; i++) {
      const op = change.ops[i]
      if (op.type === 'set' && op.path[0] === sheet.id && op.path.length === 4) {
        const [row, col] = op.path.slice(2)
        const cell = doc.get(op.val)
        this._setCell(row, col, cell)
      }
    }
    this.update()
  }

  _initialize() {
    const sheet = this.sheet
    const matrix = sheet.getMatrix()
    const N = sheet.getRowCount()
    const M = sheet.getColumnCount()
    for (let rowIdx = 0; rowIdx < N; rowIdx++) {
      const row = matrix[rowIdx]
      if (row) {
        for (let colIdx = 0; colIdx < M; colIdx++) {
          const cell = row[colIdx]
          if (cell) {
            this._setCell(rowIdx, colIdx, cell)
          }
        }
      }
    }
    this.update()
  }

  setCell(rowIdx, colIdx, cellStr) {
    this.nrows = Math.max(this.nrows, rowIdx+1)
    this.ncols = Math.max(this.ncols, colIdx+1)
    this._setCell(rowIdx, colIdx, cellStr)

    this.update()
  }

  _setCell(rowIdx, colIdx, cell) {
    // TODO: consolidate this. Would be better to have
    // a SheetScope with dedicated API to retrieve
    // data from the table
    const data = this._data
    if (!data[rowIdx]) data[rowIdx] = []
    let row = data[rowIdx]
    let oldVal = row[colIdx]
    if (oldVal instanceof Expression) {
      this._removeExpression(oldVal.id)
    }
    if (!cell) return
    let newVal
    if (cell.isLiteral()) {
      newVal = cell.content
    } else {
      // TODO: we get 'mini' cells here, to follow our decisions
      // this is a node containing the AST, which can be added
      // to the engine
      let expr = parse(cell.getExpression())
      let entry = this._addExpression(expr)
      newVal = entry
      entry.on('value:updated', () => {
        cell.setValue(entry.getValue())
      })
    }
    // store the value in the scope
    row[colIdx] = newVal
  }

}