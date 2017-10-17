import { NodeComponent } from 'substance'

export default class SheetCell extends NodeComponent {
  didMount() {
    const cell = this.props.node
    cell.on('issue:changed', this.rerender, this)
  }

  dispose() {
    const cell = this.props.node
    cell.off(this)
  }

  render($$) {
    const cell = this.props.node
    const issueManager = this.context.issueManager
    // TODO: implement this fully
    let el = $$('div').addClass('sc-sheet-cell')

    let cellIssues = issueManager.getCellIssues(cell.id)
    if(cellIssues.length > 0) {
      el.addClass('sm-issue sm-error')
    }

    el.append(this._renderContent($$, cell))
    return el
  }

  _renderContent($$, cell) {
    // TODO: this should be delegated to components
    return $$('div').addClass('sc-text-content').text(cell.text())
  }

  getContent() {
    return this.props.node.getText()
  }

}
