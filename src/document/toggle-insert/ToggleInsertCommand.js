import { Command } from 'substance'

class ToggleInsertCommand extends Command {

  getCommandState(params) {
    const { selection } = params
    let editorSession = this._getEditorSession(params)
    let doc = editorSession.getDocument()
    let newState = {
      disabled: true,
      active: false
    }
    if (selection.isPropertySelection() && selection.start.offset === 0 && selection.end.offset === 0) {
      let content = doc.get(selection.getPath())
      let type = doc.get(selection.getPath()[0]).type
      if (content.length === 0 && type === 'paragraph') {
        newState.disabled = false
      }
    }
    return newState
  }

}

export default ToggleInsertCommand
