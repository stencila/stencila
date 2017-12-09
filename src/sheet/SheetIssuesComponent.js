import { Component, forEach, ScrollPane } from 'substance'
import CellIssueComponent from './CellIssueComponent'

export default class SheetIssuesComponent extends Component {

  didMount() {
    const issueManager = this.context.issueManager
    issueManager.on('issues:changed', this._onIssuesChnage, this)
    const cellId = this.props.cellId
    if(cellId) {
      this.refs.scrollPane.scrollTo('[data-key="' + cellId + '"]')
    }
  }

  dispose() {
    const issueManager = this.context.issueManager
    issueManager.off(this)
  }

  render($$) {
    const issueManager = this.context.issueManager
    const issues = issueManager.getAllIssues()
    const cellId = this.props.cellId
    let el = $$('div').addClass('sc-sheet-issues-list')
    let scrollPane = $$(ScrollPane).ref('scrollPane')
    forEach(issues, (issue) => {
      let highlighted = issue.cellId === cellId
      scrollPane.append(this._renderIssue($$, issue, highlighted))
    })
    el.append(scrollPane)

    return el
  }

  _renderIssue($$, issue, highlighted) {
    return $$(CellIssueComponent, { issue, highlighted: highlighted })
  }

  _onIssuesChnage() {
    const issueManager = this.context.issueManager
    const hasIssues = issueManager.hasAnyIssues()
    if(hasIssues) {
      this.rerender()
    } else {
      this._close()
    }
  }

  _close() {
    let sheetEditor = this.context.app.getSheetEditor()
    if (sheetEditor) {
      sheetEditor.toggleContext('sheet-issues')
    }
  }
}