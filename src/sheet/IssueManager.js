import { ArrayTree, EventEmitter } from 'substance'
import { clone } from 'lodash-es'
import CellIssue from '../shared/CellIssue'

const SEVERITY_MAP = {
  0: 'info',
  1: 'warning',
  2: 'error',
  3: 'test-failed',
  4: 'test-passed'
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
    this.editorSession.onUpdate('document', this._onDocumentChange, this, {
      resource: 'document'
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

  removeByColumn(colId, issue) {
    this._byColumn.remove(colId, issue)
    this._byCell.remove(issue.cellId, issue)
    this._bySeverity.remove(SEVERITY_MAP[issue.severity], issue)
    this._byRow.remove(issue.rowId, issue)
    this._byKey.remove(issue.scope, issue)

    let rowCell = this.doc.get(issue.rowId)
    rowCell.emit('issue:changed')
  }

  removeByRow(rowId, issue) {
    this._byRow.remove(rowId, issue)
    this._byColumn.remove(issue.colId, issue)
    this._byCell.remove(issue.cellId, issue)
    this._bySeverity.remove(SEVERITY_MAP[issue.severity], issue)
    this._byKey.remove(issue.scope, issue)

    let columnHeader = this.doc.getColumnForCell(issue.colId)
    columnHeader.emit('issue:changed')
  }

  clear(key) {
    let issues = this._byKey.get(key)
    let issuesArr = clone(issues)
    issuesArr.forEach((issue) => {
      this.remove(key, issue)
    })
  }

  clearCellEngineIssues(cellId) {
    const cellIssues = this._byCell.get(cellId)
    cellIssues.forEach(issue => {
      if(issue.scope === 'engine') {
        this.remove('engine', issue)
      }
    })
  }

  clearByColumn(colId) {
    let issues = this._byColumn.get(colId)
    let issuesArr = clone(issues)
    issuesArr.forEach((issue) => {
      this.removeByColumn(colId, issue)
    })
    this.emit('issues:changed')
  }

  clearByRow(rowId) {
    let issues = this._byRow.get(rowId)
    let issuesArr = clone(issues)
    issuesArr.forEach((issue) => {
      this.removeByRow(rowId, issue)
    })
    this.emit('issues:changed')
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
      info: index.get('info').length,
      failed: index.get('failed').length,
      passed: index.get('passed').length
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
    const column = this.doc.getColumnForCell(issue.cellId)
    const cell = this.doc.get(issue.cellId)
    const rowId = cell.parentNode.id
    issue.colId = column.id
    issue.rowId = rowId
    issue.scope = key
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
            this.selectedCell = null
          }
        }
      }
    }
  }

  _onDocumentChange(change) {
    const columnsUpdate = change.updated.columns
    const columnsChanged = columnsUpdate !== undefined
    if(columnsChanged) {
      let deletedColumns = []
      columnsUpdate.ops.forEach(op => {
        if(op.diff.type === 'delete') {
          deletedColumns.push(op.diff.val)
        }
      })
      deletedColumns.forEach(colId => {
        this.clearByColumn(colId)
      })
    }

    let rowsChanged = false
    const updatedKeys = Object.keys(change.updated)
    updatedKeys.forEach(key => {
      if(key.indexOf('row') > -1) rowsChanged = true
    })
    if(rowsChanged) {
      let deletedRows = []
      const data = change.updated.data
      if(data) {
        data.ops.forEach(op => {
          if(op.diff.type === 'delete' && op.diff.val.indexOf('row') > -1) {
            deletedRows.push(op.diff.val)
          }
        })
      }

      deletedRows.forEach(rowlId => {
        this.clearByRow(rowlId)
      })
    }
    const stateUpdate = change.updated.setState
    const stateUpdated = stateUpdate !== undefined
    if(stateUpdated) {
      stateUpdate.forEach(cellId => {
        const cell = this.doc.get(cellId)
        if(cell.state) {
          this.clearCellEngineIssues(cellId)
          const hasErrors = cell.state.hasErrors()
          if(hasErrors) {
            cell.state.messages.forEach((err) => {
              const error = new CellIssue(cell.id, 'engine', err.message, 2)
              this.add('engine', error)
            })
          }
        }
      })
    }
  }
}

export default IssueManager
