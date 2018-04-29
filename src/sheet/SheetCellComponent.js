import { NodeComponent } from 'substance'
import ValueComponent from '../shared/ValueComponent'
import { isExpression, getError, getValue } from '../shared/cellHelpers'

export default class SheetCellComponent extends NodeComponent {

  didMount() {
    super.didMount()

    const cell = this.props.node
    cell.on('issue:changed', this.rerender, this)
  }

  dispose() {
    super.dispose()

    const cell = this.props.node
    cell.off(this)
  }

  render($$) {
    const cell = this.props.node
    let el = $$('div').addClass('sc-sheet-cell')
    let error = getError(cell)

    if (error) {
      el.append(
        $$('div').addClass('se-error').append(
          getError(cell).message
        )
      )
      el.addClass('sm-issue sm-error')
    } else {
      el.append(this._renderContent($$, cell))
    }

    return el
  }

  _renderContent($$, cell) {
    const text = cell.text()
    const isExpressionCell = isExpression(text)
    const value = getValue(cell)
    if(isExpressionCell) {
      const valueEl = $$(ValueComponent, value).ref('value')
      return $$('div').addClass('sc-text-content').append(valueEl)
    } else {
      return $$('div').addClass('sc-text-content').text(text)
    }
  }

  getContent() {
    return this.props.node.getText()
  }
}
