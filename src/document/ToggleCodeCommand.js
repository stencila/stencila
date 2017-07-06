import { Command } from 'substance'

export default class ToggleCodeCommand extends Command {

  /*
    Always enabled
  */
  getCommandState() {
    return {
      disabled: false,
      active: false
    }
  }

  /*
    Returns all cell components found in the document
  */
  _getCellComponents(params) {
    let editor = params.editorSession.getEditor()
    return editor.findAll('.sc-cell')
  }

  execute(params) {
    let cellComponents = this._getCellComponents(params)
    cellComponents.forEach((cellComponent) => {
      cellComponent.extendState({
        showMenu: false,
        showCode: this.config.showCode
      })
    })
    params.editorSession.setSelection(null)
  }
}
