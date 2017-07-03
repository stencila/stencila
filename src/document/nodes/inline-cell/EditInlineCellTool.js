import { Tool } from 'substance'

/*
  TODO: Use Substance TextInput component
*/
class EditInlineCellTool extends Tool {

  getExpressionPath() {
    return [ this.props.commandState.nodeId ].concat('expression')
  }

  render($$) {
    let Input = this.getComponent('input')
    let el = $$('div').addClass('sc-edit-inline-cell-tool')
    let expressionPath = this.getExpressionPath()
    el.append(
      'Output ',
      $$(Input, {
        type: 'text',
        path: expressionPath,
        placeholder: 'Type Mini Expression (e.g. 5 * 5)'
      })
    )
    return el
  }
}

export default EditInlineCellTool
