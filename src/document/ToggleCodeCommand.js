import { Command } from 'substance'

export default class ToggleCodeCommand extends Command {

  _getNodeComponent(editorSession, nodeId) {
    let editor = editorSession.getEditor()
    return editor.find(`[data-id=${nodeId}]`)
  }

  getCommandState({ editorSession, selection}) {
    let sel = selection
    
    let node = sel.getNode()
    let state = { nodeId: node.id, disabled: false }
    let comp = this._getNodeComponent(editorSession, node.id)

    if (comp.state.hideCode && this.config.hideCode) {
      state.disabled = true
    }
    return state
  }

  execute({ commandState, editorSession }) {
    const { nodeId, disabled } = commandState
    if (!disabled) {
      let comp = this._getNodeComponent(editorSession, nodeId)
      if (this.config.hideCode && !comp.state.hideCode) {
        comp.extendState({ hideCode: true})
      } else if (!this.config.hideCode && comp.state.hideCode) {
        comp.extendState({ hideCode: false}) // aka show code
      }
    }
  }
}
