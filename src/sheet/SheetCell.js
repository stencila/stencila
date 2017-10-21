import { NodeComponent } from 'substance'
import CellValueComponent from '../shared/CellValueComponent'

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
    if(textValue.indexOf('=test') > -1) this.setFakeState(cell)
    if(this.props.mode === 'maximum') {
      //const value = isTextCell ? textValue : this.getResponse()
      let valueEl = $$('div').addClass('sc-cell-value').append(
        $$(CellValueComponent, {cell: cell}).ref('value')
      )
      if(!isTextCell) valueEl.addClass('sm-response-value')

      const source = !isTextCell ? textValue : ' '
      return $$('div').addClass('se-function-cell').append(
        valueEl,
        $$('div').addClass('sc-equation').append(source)
      )
    }
    // else if (this.props.mode === 'minimum') {
    //   if(!isTextCell) textValue = this.getResponse() || ' '
    //   return $$('div').addClass('sc-text-content').text(textValue)
    // }
    else {
      if(cell.state) {
        return $$('div').addClass('sc-text-content').append(
          $$(CellValueComponent, {cell: cell}).ref('value')
        )
      } else {
        return $$('div').addClass('sc-text-content').text(textValue)
      }      
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
