import { NodeComponent } from 'substance'

export default class SheetCell extends NodeComponent {

  render($$) {
    const cell = this.props.node
    // TODO: implement this fully
    let el = $$('div').addClass('sc-sheet-cell')
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