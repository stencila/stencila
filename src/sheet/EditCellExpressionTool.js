import { ToggleTool } from 'substance'
import SheetCellEditor from './SheetCellEditor'

export default class EditCellExpressionTool extends ToggleTool {

  render($$) {
    let editorSession = this.context.editorSession
    let doc = editorSession.getDocument()
    let node = doc.get(this.props.commandState.cellId)

    let el = $$('div').addClass('sc-edit-cell-expression-tool').append(
      $$('div').addClass('se-function-icon').append(
        $$('em').append(
          'ƒ',
          $$('sub').append('x')
        )
      ),
      $$(SheetCellEditor, {
        name: 'cell-expression-tool-editor',
        node: node,
        editorSession: this.context.cellEditorSession
      })
        .on('enter', this._onCellEditorEnter)
        .on('escape', this._onCellEditorEscape)
        .ref('cellEditor')
    )
    return el
  }

  _getSheet() {
    return this.context.editorSession.getDocument()
  }

  _onCellEditorEnter() {
    this.send('onCellEditorEnter')
  }

  _onCellEditorEscape() {
    this.send('onCellEditorEscape')
  }
}
