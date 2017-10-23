import { NodeComponent } from 'substance'
import CellValueComponent from '../shared/CellValueComponent'
import { isExpression } from '../shared/cellHelpers'

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
    const issueManager = this.context.issueManager
    let el = $$('div').addClass('sc-sheet-cell')

    let cellIssues = issueManager.getCellIssues(cell.id)
    if(cellIssues.length > 0) {
      el.addClass('sm-issue sm-error')
    }

    el.append(this._renderContent($$, cell))
    return el
  }

  _renderContent($$, cell) {
    let text = cell.text()
    let isExpressionCell = isExpression(text)

    if(isExpressionCell) {
      if(this.props.mode === 'maximum') {
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

  getContent() {
    return this.props.node.getText()
  }

  getResponse() {
    return '24.2324'
  }

  setFakeState(cell) {
    cell.state = {
      hasValue: function() {
        return true
      },
      getValue: function() {
        return {
          type: 'test',
          passed: true,
          message: 'Sorry, my friend'
        }
      },
      hasErrors: function() {
        return false
      }
    }
  }

}
