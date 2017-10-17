import { EventEmitter, forEach, map } from 'substance'

// EXPERIMENTAL: a first shot for a linter running
// in the 'background' checking every cell
// and rescheduling checks when cells change
export default class SheetLinter extends EventEmitter {

  constructor(sheet, editorSession) {
    super()

    this.sheet = sheet
    this.editorSession = editorSession

    this.queue = []
    this.issues = []
    this.state = 'initial'

    // TODO: need to rethink this when we want to use this
    // for a different underlying implementation, e.g an embedded table
    this._sheetNode = sheet.getRootNode()
    this._doc = this._sheetNode.getDocument()

    editorSession.onRender('document', this._onDocumentChange, this, { path: [this._sheetNode.id]})
  }

  start() {
    if (this._isInitial()) {
      this.warmup()
    }
    if (this._isStopped()) {
      this._setState('running')
      this.step()
    }
  }

  restart() {
    this._setState('initial')
    this.start()
  }

  warmup() {
    this.queue = []
    const sheet = this.sheet
    let N = sheet.getRowCount()
    let M = sheet.getColumnCount()
    for (let i = 0; i < N; i++) {
      for (let j = 0; j < M; j++) {
        let cell = sheet.getCell(i, j)
        this.queue.push(cell)
      }
    }
    this._setState('stopped')
  }

  step() {
    if (this.queue.length > 0) {
      this.action()
      if (this._isRunning()) {
        setTimeout(() => {
          this.step()
        })
      }
    } else {
      this._setState('stopped')
    }
  }

  action() {
    let next = this.queue.shift()
    if (!next) return
    switch(next.type) {
      case 'cell': {
        // Note: retrieving the cell
        if (this.sheet.contains(next.id)) {
          this.validateCell(next)
        }
        break
      }
      default:
        //
    }

  }

  validateCell(cell) {
    let sheet = this.sheet
    checkType(cell, sheet, this)
  }

  hasIssues() {
    return this.issues.length > 0
  }

  getIssues() {
    return this.issues.slice()
  }

  getNumberOfIssues() {
    return this.issues.length
  }

  hasErrors() {
    for (let i = 0; i < this.issues.length; i++) {
      if (this.issues[i].isError()) return true
    }
    return false
  }

  addIssue(issue) {
    // console.log('Issue in cell', issue.cell, issue)
    this.issues.push(issue)
    this._updateCommandStates()
    // send an issue to the node
    if(issue.cell) {
      let cell = issue.cell
      cell._issue = issue
      cell.emit('issue:changed')
    }
    this.emit('issues:changed')
  }

  _updateCommandStates() {
    // HACK: need to re-flow the editor session, as the command states
    // should be re-evaluated and toolbars rerendered
    const editorSession = this.editorSession
    editorSession._setDirty('commandStates')
    editorSession.performFlow()
  }

  _onDocumentChange(change) {
    let needsUpdate = false
    const sheet = this.sheet
    const N = sheet.getRowCount()
    // TODO we need to detect all relevant changes
    // for scheduling checks
    let cells = {}
    let cols = {}
    for (let i = 0; i < change.ops.length; i++) {
      let op = change.ops[i]
      let nodeId = op.path[0]
      let node = this._doc.get(nodeId)
      if (!node) continue
      // revalidate on all cell changes
      if (node.type === 'cell') {
        cells[node.id] = node
        needsUpdate = true
      } else if (node.type === 'col') {
        if (op.path[2] === 'type') {
          cols[node.id] = node
          needsUpdate = true
        }
      }
    }
    // extract all cell ids for changed columns
    forEach(cols, (column) => {
      let colIdx = sheet.getColumnIndex(column)
      for (let i = 0; i < N; i++) {
        let cell = sheet.getCell(i, colIdx)
        cells[cell.id] = cell
      }
    })
    if (needsUpdate) {
      let newChecks = map(cells)
      // revalidate existing
      let revalidations = []
      this.issues.forEach((issue) => {
        if (issue.isCellIssue() && !cells[issue.cell.id]) {
          revalidations.push(issue.cell)
        }
      })
      this.queue = newChecks.concat(revalidations).concat(this.queue)
      this.issues = []
      this.emit('issues:changed')
      this.start()
      // need to
      this.editorSession.postpone(() => {
        this._updateCommandStates()
      })
    }
  }

  _isRunning() {
    return this.state === 'running'
  }

  _isInitial() {
    return this.state === 'initial'
  }

  _isStopped() {
    return this.state === 'stopped'
  }

  _setState(state) {
    // console.log(`SheetLinter: ${this.state} -> ${state}`)
    this.state = state
    this.emit(state)
  }

}

// EXPERIMENTAL:
function checkType(cell, sheet, linter) {
  let type = sheet.getCellType(cell)
  let str = cell.textContent
  let wrongType = false
  switch (type) {
    case 'integer': {
      wrongType = (isNaN(str) || !isInt(str))
      break
    }
    case 'number': {
      wrongType = isNaN(str)
      break
    }
    case 'boolean': {
      wrongType = (str !== 'false' && str !== 'true')
      break
    }
    case 'string':
    case 'any':
      // nothing
      break
    default:
      // nothing
  }
  if (wrongType) {
    let expected = type
    let actual = autoDetectType(str)
    linter.addIssue(new CellTypeError(cell, expected, actual))
  }
}

// TODO: these should go into cellHelpers
function isInt(str) {
  return (parseInt(str, 10) == str) // eslint-disable-line eqeqeq
}

// TODO: we need to discuss if we want to
// allow '1' as float, or if we want to force '1.0' when type is float
function isFloat(str) {
  return (str.indexOf('.') !== -1)
}

function autoDetectType(str) {
  // numbers
  if (!isNaN(str)) {
    if (isInt(str)) {
      return 'integer'
    }
    if (isFloat(str)) {
      return 'float'
    }
    return 'number'
  }
  // boolean
  if (str === 'false' || str === 'true') {
    return 'boolean'
  }
  // TODO: add more
  return 'string'
}


class CellTypeError {

  constructor(cell, expected, actual) {
    this.cell = cell
    this.expected = expected
    this.actual = actual
  }

  getKey() {
    return `${this.type}#${this.cell.id}`
  }

  get type() {
    return 'wrong-type'
  }

  isError() {
    return true
  }

  isCellIssue() {
    return true
  }

  getMessage() {
    return `Cell content is of wrong type. Expected a ${this.expected}, but it is a ${this.actual}`
  }

}
