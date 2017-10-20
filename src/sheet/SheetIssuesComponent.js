import { Component, forEach, ScrollPane, stopAndPrevent } from 'substance'

export default class SheetIssuesComponent extends Component {

  didMount() {
    const issueManager = this.context.issueManager
    issueManager.on('issues:changed', this.rerender, this)
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
}

class CellIssueComponent extends Component {
  render($$) {
    const issue = this.props.issue
    const highlighted = this.props.highlighted
    const doc = this.context.doc
    let el = $$('div').addClass('sc-cell-issue')
      .attr('data-key', issue.cellId)
    let severity = 'info'
    if(issue.severity === 1) {
      severity = 'warning'
    } else if (issue.severity === 2) {
      severity = 'error'
    }
    el.addClass(`sm-${severity}`)
    if(highlighted) {
      el.addClass('sm-highlighted')
    }
    let cellName = $$('div').addClass('se-cell-name')
      .text(doc.getCellLabel(issue.cellId))
    let title = $$('div').addClass('se-title')
      .text(this.getLabel(`title:${severity}`))
    let message = $$('div').addClass('se-message')
      .text(issue.message)
    el.append(
      cellName,
      title,
      message
    )
    el.on('click', this._onClick)
    return el
  }

  _onClick(e) {
    stopAndPrevent(e)
    let issue = this.props.issue
    let editor = this.context.editor
    editor.setSelectionOnCell(issue.cellId)
  }
}
