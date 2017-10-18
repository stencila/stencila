import { NodeComponent } from 'substance'

class SheetRowHeader extends NodeComponent {
  didMount() {
    const cell = this.props.node
    cell.on('issue:changed', this.rerender, this)
  }

  dispose() {
    const cell = this.props.node
    cell.off(this)
  }

  render($$) {
    const rowIdx = this.props.rowIdx
    const node = this.props.node
    const issueManager = this.context.issueManager

    let th = $$('th')
      .attr('data-col', rowIdx)
      .addClass('sc-column-header')
      .text(String(rowIdx + 1))

    let cellIssues = issueManager.getRowIssues(node.id)
    if(cellIssues.length > 0) {
      th.addClass('sm-issue sm-error')
    }

    return th
  }
}

export default SheetRowHeader
