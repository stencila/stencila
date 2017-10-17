import { ArrayTree } from 'substance'

class IssueManager {
  constructor(context) {
    if (!context.editorSession) {
      throw new Error('EditorSession required.')
    }

    this.editorSession = context.editorSession
    this.doc = this.editorSession.getDocument()
    this._byKey = new ArrayTree()
    this._byCell = new ArrayTree()
    this._byColumn = new ArrayTree()
  }

  add(key, issue) {
    let column = this.doc.getColumnForCell(issue.cellId)
    this._byKey.add(key, issue)
    this._byCell.add(issue.cellId, issue)
    this._byColumn.add(column.id, issue)
    this._notifyObservers(issue.cellId)
  }

  remove(key, issue) {
    this._byKey.remove(key, issue)
    this._byCell.remove(issue.cellId, issue)
    let column = this.doc.getColumnForCell(issue.cellId)
    this._byColumn.remove(column.id, issue)
    this._notifyObservers(issue.cellId)
  }

  clear(key) {
    let issues = this._byKey.get(key)
    issues.forEach((issue) => {
      this.remove(key, issue)
    })
  }

  getIssues(key) {
    return this._byKey.get(key)
  }

  getCellIssues(cellId) {
    return this._byCell.get(cellId)
  }

  getColumnIssues(columnId) {
    return this._byColumn.get(columnId)
  }

  getNumberOfIssues(key) {
    let issues = this._byKey.get(key)
    return issues.length
  }

  hasIssues(key) {
    let issues = this._byKey.get(key)
    return issues.length > 0
  }

  hasErrors(key) {
    let issues = this._byKey.get(key)
    for (let i = 0; i < issues.length; i++) {
      if (issues[i].isError()) return true
    }
    return false
  }

  _notifyObservers(cellId) {
    let cell = this.doc.get(cellId)
    cell.emit('issue:changed')

    let columnHeader = this.doc.getColumnForCell(cellId)
    columnHeader.emit('issue:changed')
  }
}

export default IssueManager
