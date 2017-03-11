import { Tool } from 'substance'

/*
  TODO: Use Substance TextInput component
*/
class EditInlineCellTool extends Tool {

  getExpressionPath() {
    return [ this.props.node.id ].concat('expression')
  }

  render($$) {
    let Input = this.getComponent('input')
    let el = $$('div').addClass('sc-edit-inline-cell-tool')
    let expressionPath = this.getExpressionPath()
    el.append(
      $$(Input, {
        type: 'text',
        path: expressionPath,
        placeholder: 'Type Minilang...'
      })
    )
    return el
  }
}

export default EditInlineCellTool
