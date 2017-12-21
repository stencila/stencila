import { NodeComponent } from 'substance'
import CellValueComponent from '../shared/CellValueComponent'
import CellOverflowComponent from './CellOverflowComponent'
import { isExpression, getError } from '../shared/cellHelpers'
import { isOverflowable } from './sheetHelpers'

export default class SheetCell extends NodeComponent {

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
    let text = cell.text()
    let isExpressionCell = isExpression(text)

    if(isExpressionCell) {
      const displayMode = this._getDisplayMode()
      if(displayMode === 'maximum') {
        return $$('div').addClass('se-function-cell').append(
          $$(CellValueComponent, {cell: cell}).ref('value'),
          $$('div').addClass('sc-equation').append(text)
        )
      } else {
        const needsOverflow = isOverflowable(cell)
        const valueEl = $$(CellValueComponent, {cell: cell}).ref('value')
        if(needsOverflow) {
          return $$(CellOverflowComponent).append(valueEl)
        } else {
          return $$('div').addClass('sc-text-content').append(valueEl)
        }
      }
    } else {
      return $$('div').addClass('sc-text-content').text(text)
    }
  }

  getContent() {
    return this.props.node.getText()
  }

  _getDisplayMode() {
    let sheetState = this.props.node.getDocument().getState()
    return sheetState.displayMode
  }
}
