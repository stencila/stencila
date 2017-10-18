import { Component, forEach, stopAndPrevent } from 'substance'

export default class SheetIssuesComponent extends Component {

  didMount() {
    const issueManager = this.context.issueManager
    issueManager.on('issues:changed', this.rerender, this)
  }

  dispose() {
    const issueManager = this.context.issueManager
    issueManager.off(this)
  }

  render($$) {
    const issueManager = this.context.issueManager
    const issues = issueManager.getAllIssues()
    let el = $$('div').addClass('sc-sheet-issues-list')
    forEach(issues, (issue) => {
      el.append(this._renderIssue($$, issue))
    })
    return el
  }

  _renderIssue($$, issue) {
    return $$(CellIssueComponent, { issue })
  }

}

class CellIssueComponent extends Component {
  render($$) {
    const issue = this.props.issue
    const doc = this.context.doc
    let el = $$('div').addClass('sc-cell-issue')
    let severity = 'info'
    if(issue.severity === 1) {
      severity = 'warning'
    } else if (issue.severity === 2) {
      severity = 'error'
    }
    el.addClass(`sm-${severity}`)
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
    console.log('Clicked on issue', this.props.issue)
  }
}
