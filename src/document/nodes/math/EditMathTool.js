import { Tool } from 'substance'

/*
  Tool to edit math markup.

  TODO: Consider auto-detecting whether Asciimath or Tex math has been entered.
  That way we would not need to show another UI element.
*/
class EditMathTool extends Tool {

  getSourcePath() {
    return [ this.props.node.id ].concat('source')
  }

  render($$) {
    let Input = this.getComponent('input')
    let el = $$('div').addClass('sc-edit-math-tool')

    // GUARD: Return if tool is disabled
    if (this.props.disabled) {
      console.warn('Tried to render EditLinkTool while disabled.')
      return el
    }

    let sourcePath = this.getSourcePath()
    el.append(
      $$(Input, {
        type: 'text',
        path: sourcePath,
        placeholder: 'Type Asciimath'
      })
    )
    return el
  }
}

export default EditMathTool
