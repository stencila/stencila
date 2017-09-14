import { Component, forEach, stopAndPrevent } from 'substance'

export default
class SheetIssuesList extends Component {

  render($$) {
    const issues = this.props.issues
    let el = $$('div').addClass('sc-sheet-issues-list')
    forEach(issues, (issue) => {
      el.append(this._renderIssue($$, issue))
    })
  }

  _renderIssue($$, issue) {
    return $$(CellIssueComponent, { issue })
  }

}

class CellIssueComponent extends Component {
  render($$) {
    const issue = this.props.issue
    let el = $$('div').addClass('sc-cell-issue')
    el.addClass(`sm-${issue.getSeverity()}`)
    let title = $$('div').addClass('se-title')
      .text(this.getLabel(`title:${issue.type}`))
    let message = $$('div').addClass('se-message')
      .text(issue.getMessage())
    el.append(
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
