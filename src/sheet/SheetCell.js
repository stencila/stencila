import { NodeComponent } from 'substance'
import CellValueComponent from '../shared/CellValueComponent'
import { isExpression, getError } from '../shared/cellHelpers'

export default class SheetCell extends NodeComponent {

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

    if(isExpressionCell) {
      const displayMode = this._getDisplayMode()
      if(displayMode === 'maximum') {
        return $$('div').addClass('se-function-cell').append(
          $$(CellValueComponent, {cell: cell}).ref('value'),
          $$('div').addClass('sc-equation').append(text)
        )
      } else {
        return $$('div').addClass('sc-text-content').append(
          $$(CellValueComponent, {cell: cell}).ref('value')
        )
      }
    } else {
      return $$('div').addClass('sc-text-content').text(text)
    }
  }

  _getDisplayMode() {
    let sheetState = this.props.node.getDocument().getState()
    return sheetState.displayMode
  }
}
