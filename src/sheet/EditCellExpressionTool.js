import { ToggleTool } from 'substance'

export default class EditCellExpressionTool extends ToggleTool {

  render($$) {
    let el = $$('div').addClass('sc-edit-cell-expression-tool').append(
      $$('input').attr({type: 'text', placeholder: 'Enter Value or Expression'})
    )
    return el
  }
}
