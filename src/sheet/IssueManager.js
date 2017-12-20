import { ArrayTree, EventEmitter, clone, uuid } from 'substance'
import CellIssue from '../shared/CellIssue'
import { SEVERITY_NAMES, getCellState } from '../shared/cellHelpers'

export default class IssueManager extends EventEmitter {

  constructor(context) {
    super()

    if (!context.editorSession) {
      throw new Error('EditorSession required.')
    }
    this.editorSession = context.editorSession

    // TODO: we should think about which indexes
    // are really necessary, or what
    // is ok to derive dynamically from all issues
    this._issues = new Set()
    this._byType = {}

    // listening to document changes so that we can remove
    // issues that are stored on nodes from the index
    this.editorSession.onUpdate('document', this._onDocumentChange, this)
  }

  add(...issues) {
    issues.forEach((issue) => {
      this._add(issue)
    })
    this.emit('issues:changed')
  }

  remove(...issues) {
    issues.forEach((issue) => {
      this._remove(issue)
    })
    this.emit('issues:changed')
  }

  clear(type) {
    let issues = this._byType[type]
    if (issues) {
      Array.from(issues).forEach((issue) => {
        this._remove(issue)
      })
      this.emit('issues:changed')
    }
  }

  hasIssues(type) {
    let issues = this.getIssues(type)
    return issues.size > 0
  }

  getIssues(type) {
    if (type) {
      return this._byType[type] || new Set()
    } else {
      return this._issues
    }
  }

  getStats() {
    if (!this._stats) {
      this._computeStats()
    }
    return this._stats
  }

  _invalidateCache() {
    this._stats = null
  }

  _computeStats() {
    let issues = this.getIssues()
    // count issues grouped by severity
    let counters = {
      error: 0,
      warning: 0,
      info: 0
    }
    this._stats = {
      counters
    }
  }

  _add(issue) {
    if (!(issue instanceof CellIssue)) {
      issue = new CellIssue(issue)
    }
    this._issues.add(issue)
    const type = issue.type
    let byType = this._byType[type]
    if (!byType) {
      this._byType[key] = new Set()
    }
    byType.add(issue)
    this._invalidateCache()
  }

  _remove(issue) {
    const type = issue.type
    this._issues.remove(issue)
    let byType = this._byType[key]
    if (byType) {
      byType.remove(issue)
    }
    this._invalidateCache()
  }

  _onDocumentChange(change) {
    forEach(change.deleted, (node) => {
      // TODO: is it possible to generalize this?
      if (node.type === 'cell') {
        let cellState = getCellState(n)
        let issues = cellState.getIssues()
        issues.forEach((issue) => {
          this._remove(issue)
        })
      }
    })
  }

}