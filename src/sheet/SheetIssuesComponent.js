import { Component, forEach, stopAndPrevent } from 'substance'

export default
class SheetIssuesComponent extends Component {

  didMount() {
    const linter = this.props.editor.getLinter()
    linter.on('issues:changed', this.rerender, this)
  }

  dispose() {
    const linter = this.props.editor.getLinter()
    linter.off(this)
  }

  render($$) {
    const issues = this.props.editor.getIssues()
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
    let el = $$('div').addClass('sc-cell-issue')
    let severity = issue.isError() ? 'error' : 'warning'
    el.addClass(`sm-${severity}`)
    let title = $$('div').addClass('se-title')
      .text(this.getLabel(`title:${severity}`))
    let message = $$('div').addClass('se-message')
      .text(issue.message)
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
