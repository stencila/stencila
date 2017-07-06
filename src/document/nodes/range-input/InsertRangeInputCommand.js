import { InsertInlineNodeCommand, documentHelpers } from 'substance'

class InsertRangeInputCommand extends InsertInlineNodeCommand {

  isDisabled (params) {
    let text = this._getText(params)
    if (isNaN(text)) {
      return true
    } else {
      return super.isDisabled(params)
    }
  }

  _getText (params) {
    let editorSession = this._getEditorSession(params)
    let doc = editorSession.getDocument()
    return documentHelpers.getTextForSelection(
      doc,
      params.selection
    )
  }

  createNodeData (tx, params) {
    let text = this._getText(params) || '100'
    var number = Number(text)
    return {
      type: 'range-input',
      name: '',
      min: number / 2,
      max: number + number / 2,
      step: 1,
      value: number,
      selectedIndex: 0
    }
  }
}

export default InsertRangeInputCommand
