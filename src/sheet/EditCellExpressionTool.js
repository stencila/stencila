import { ToggleTool } from 'substance'
// import SheetCellEditor from './SheetCellEditor'
import { getCellState } from '../shared/cellHelpers'
import CodeEditor from '../shared/CodeEditor'


export default class EditCellExpressionTool extends ToggleTool {

  render($$) {
    let editorSession = this.context.editorSession
    let doc = editorSession.getDocument()
    let node = doc.get(this.props.commandState.cellId)
    let cellState = getCellState(node)

    let el = $$('div').addClass('sc-edit-cell-expression-tool').append(
      $$('div').addClass('se-function-icon').append(
        $$('em').append(
          'ƒ',
          $$('sub').append('x')
        )
      ),

      $$(CodeEditor, {
        path: node.getPath(),
        excludedCommands: this._getBlackListedCommands(),
        tokens: cellState.tokens
      }).ref('cellEditor')
        .on('enter', this._onCodeEditorEnter)
        .on('escape', this._onCodeEditorEscape)
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

  // TODO: find better way
  _getBlackListedCommands() {
    const commandGroups = this.context.commandGroups
    let result = []
    ;['annotations', 'insert', 'prompt', 'text-types'].forEach((name) => {
      if (commandGroups[name]) {
        result = result.concat(commandGroups[name])
      }
    })
    return result
  }
}
