import { ToolPanel, Component, Toolbar } from 'substance'

export default class ExpressionBar extends ToolPanel {

  render($$) {
    let el = $$('div').addClass('sc-expression-bar')
    el.append(
      $$(ExpressionBarEditor),
      $$(Toolbar, {
        toolPanel: this.props.toolPanel
      })
    )
    return el
  }

}

/*
  TODO: make this update on selection change, and always show the cell editor
  for the anchor cell.
*/
class ExpressionBarEditor extends Component {
  render($$) {
    let el = $$('div').addClass('sc-expression-bar-editor')
    el.append('TODO: edit expression here')
    return el
  }
}
