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
    let textValue = cell.text()
    const isTextCell = textValue.charAt(0) !== '='
    if(this.props.mode === 'maximum') {
      const value = isTextCell ? textValue : this.getResponse()
      let valueEl = $$('div').addClass('sc-cell-value').append(value)
      if(!isTextCell) valueEl.addClass('sm-response-value')

      const source = !isTextCell ? textValue : ' '
      return $$('div').addClass('se-function-cell').append(
        valueEl,
        $$('div').addClass('sc-equation').append(source)
      )
    } else if (this.props.mode === 'minimum') {
      if(!isTextCell) textValue = this.getResponse() || ' '
      return $$('div').addClass('sc-text-content').text(textValue)
    } else {
      return $$('div').addClass('sc-text-content').text(textValue)
    }
  }

  getContent() {
    return this.props.node.getText()
  }

  getResponse() {
    return '24.2324'
  }

}
