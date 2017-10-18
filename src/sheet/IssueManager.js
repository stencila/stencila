import { ArrayTree, EventEmitter } from 'substance'
import { clone } from 'lodash-es'

const SEVERITY_MAP = {
  0: 'info',
  1: 'warning',
  2: 'error'
}

class IssueManager extends EventEmitter {
  constructor(context) {
    super(context)

    if (!context.editorSession) {
      throw new Error('EditorSession required.')
    }

    this.editorSession = context.editorSession
    this.doc = this.editorSession.getDocument()
    this.selectedCell = null
    this.editorSession.on('render', this._onSelectionChange, this, {
      resource: 'selection'
    })
    this._byKey = new ArrayTree()
    this._bySeverity = new ArrayTree()
    this._byCell = new ArrayTree()
    this._byColumn = new ArrayTree()
    this._byRow = new ArrayTree()
  }

  add(key, issue) {
    this._add(key, issue)
    this.emit('issues:changed')
  }

  set(key, issues) {
    issues.forEach(issue => {
      this._add(key, issue)
    })
    this.emit('issues:changed')
  }

  remove(key, issue) {
    this._byKey.remove(key, issue)
    this._byCell.remove(issue.cellId, issue)
    let column = this.doc.getColumnForCell(issue.cellId)
    let cell = this.doc.get(issue.cellId)
    let rowId = cell.parentNode.id
    this._byColumn.remove(column.id, issue)
    this._byRow.remove(rowId, issue)
    this._bySeverity.remove(SEVERITY_MAP[issue.severity], issue)
    this._notifyObservers(issue.cellId)
  }

  clear(key) {
    let issues = this._byKey.get(key)
    let issuesArr = clone(issues)
    issuesArr.forEach((issue) => {
      this.remove(key, issue)
    })
  }

  getIssues(key) {
    return this._byKey.get(key)
  }

  getAllIssues() {
    let index = this._byKey
    let scopes = Object.keys(index)
    let issues = []
    scopes.forEach(scope => {
      issues = issues.concat(index[scope])
    })

    return issues
  }

  getCellIssues(cellId) {
    return this._byCell.get(cellId)
  }

  getColumnIssues(columnId) {
    return this._byColumn.get(columnId)
  }

  getRowIssues(rowId) {
    return this._byRow.get(rowId)
  }

  getNumberOfIssues(key) {
    let issues = this._byKey.get(key)
    return issues.length
  }

  getStats() {
    let index = this._bySeverity
    return {
      errors: index.get('error').length,
      warnings: index.get('warning').length,
      info: index.get('info').length
    }
  }

  hasIssues(key) {
    let issues = this._byKey.get(key)
    return issues.length > 0
  }

  hasAnyIssues() {
    let index = this._byKey
    let scopes = Object.keys(index)
    let hasIssues = scopes.find(scope => {
      return index[scope].length > 0
    })

    return hasIssues
  }

  cellHasIssue(cellId) {
    return this._byCell.get(cellId).length > 0 ? true : false
  }

  hasErrors(key) {
    let issues = this._byKey.get(key)
    for (let i = 0; i < issues.length; i++) {
      if (issues[i].isError()) return true
    }
    return false
  }

  _add(key, issue) {
    let column = this.doc.getColumnForCell(issue.cellId)
    let cell = this.doc.get(issue.cellId)
    let rowId = cell.parentNode.id
    this._byKey.add(key, issue)
    this._byCell.add(issue.cellId, issue)
    this._byColumn.add(column.id, issue)
    this._byRow.add(rowId, issue)
    this._bySeverity.add(SEVERITY_MAP[issue.severity], issue)
    this._notifyObservers(issue.cellId)
  }

  _notifyObservers(cellId) {
    let cell = this.doc.get(cellId)
    cell.emit('issue:changed')

    let columnHeader = this.doc.getColumnForCell(cellId)
    columnHeader.emit('issue:changed')

    let rowCell = this.doc.get(cell.parentNode.id)
    rowCell.emit('issue:changed')
  }

  _onSelectionChange(sel) {
    if(sel.type === 'custom' && sel.customType === 'sheet') {
      if(sel.data.type === 'range') {
        const anchorCol = sel.data.anchorCol
        const anchorRow = sel.data.anchorRow
        let doc = this.doc
        let cell = doc.getCell(anchorRow, anchorCol)
        let cellId = cell.id

        if(this.selectedCell !== cellId) {
          const hasIssue = this.cellHasIssue(cellId)
          if(hasIssue) {
            this.emit('issue:focus', cellId)
            this.selectedCell = cellId
          } else if (this.selectedCell) {
            this.emit('issue:focus', null)
            this.selectedCell = null
          }
        }
      }
    }
  }
}

export default IssueManager
