import { Component, stopAndPrevent } from 'substance'

export default class CellIssueComponent extends Component {
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
    } else if (issue.severity === 3) {
      severity = 'failed'
    } else if (issue.severity === 4) {
      severity = 'passed'
    }
    el.addClass(`sm-${severity}`)
    if(highlighted) {
      el.addClass('sm-highlighted')
    }
    let cellName = $$('div').addClass('se-cell-name')
      .text(doc.getCellLabel(issue.cellId))
    let content = this.getLabel(`title:${severity}`) + ': ' + issue.message

    el.append(
      cellName,
      $$('div').addClass('se-content').text(content)
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
