import { EventEmitter, forEach, map } from 'substance'
import CellIssue from '../shared/CellIssue'

// EXPERIMENTAL: a first shot for a linter running
// in the 'background' checking every cell
// and rescheduling checks when cells change
export default class SheetLinter extends EventEmitter {

  constructor(sheet, editorSession) {
    super()

    this.sheet = sheet
    this.editorSession = editorSession
    this.issueManager = editorSession.getManager('issue-manager')

    this.queue = []
    this.issues = []
    this.state = 'initial'

    // TODO: need to rethink this when we want to use this
    // for a different underlying implementation, e.g an embedded table
    this._sheetNode = sheet.getRootNode()
    this._doc = this._sheetNode.getDocument()

    editorSession.onRender('document', this._onDocumentChange, this, { path: [this._sheetNode.id]})
  }

  dispose() {
    super.dispose()
    this.editorSession.off(this)
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
    this.firstShot = true
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
      if(!this.firstShot) {
        this.issueManager.set('linter', this.issues)
        this.issues = []
      }
      this.firstShot = false
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

  addIssue(issue) {
    if(this.firstShot) {
      this.issueManager.add('linter', issue)
    } else {
      this.issues.push(issue)
    }
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
      let issues = this.issueManager.getIssues('linter')
      issues.forEach((issue) => {
        if (issue.isCellIssue() && !cells[issue.cellId]) {
          let cell = this._doc.get(issue.cellId)
          cells[issue.cellId] = cell
          revalidations.push(cell)
        }
      })

      this.queue = newChecks.concat(revalidations).concat(this.queue)
      this.issueManager.clear('linter')

      if(this.firstShot) {
        this.restart()
      } else {
        this.start()
      }
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
    this.state = state
    this.emit(state)
  }

}

// EXPERIMENTAL:
function checkType(cell, sheet, linter) {
  let str = cell.text()

  // Do not check cell type if it has no content!
  if (!str) return

  let type = sheet.getCellType(cell)
  // HACK: do not test dynamic expressions for now
  if (/^\s*=/.exec(str)) return

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
    let msg = `Cell content is of wrong type. Expected a ${expected}, but it is a ${actual}`
    //linter.addIssue(new CellTypeError(cell, expected, actual))
    linter.addIssue(new CellIssue(cell.id, 'linter', msg, 2))
  }
}

// TODO: these should go into cellHelpers
function isInt(str) {
  return (parseInt(str, 10) == str) // eslint-disable-line eqeqeq
}

function autoDetectType(str) {
  // numbers
  if (!isNaN(str)) {
    if (isInt(str)) {
      return 'integer'
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
