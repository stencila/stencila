import { SwitchTextTypeCommand } from 'substance'

class MinimalSwitchTextTypeCommand extends SwitchTextTypeCommand {

  /*
    Disabled unless cursor at first position in non-empty text-node
  */
  isDisabled(params) {
    const {selection, surface} = params
    let editorSession = this._getEditorSession(params)
    let doc = editorSession.getDocument()

    if (!surface || !surface.isEnabled() || selection.isNull()) {
      return true
    }

    let enabled = false
    if (selection.isPropertySelection() && selection.start.offset === 0 && selection.end.offset === 0) {
      let content = doc.get(selection.getPath())
      if (content.length > 0) enabled = true
    }
    return !enabled
  }

}

export default MinimalSwitchTextTypeCommand
