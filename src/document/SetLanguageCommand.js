import { Command } from 'substance'

export default class SetLanguageCommand extends Command {

  _getNodeComponent(editorSession, nodeId) {
    let editor = editorSession.getEditor()
    return editor.find(`[data-id=${nodeId}]`)
  }

  getCommandState({ selection }) {
    let sel = selection
    let node = sel.getNode()
    let state = { nodeId: node.id }

    // Disable when language already set
    if (node.attr('language') === this.config.language) {
      state.disabled = true
    } else {
      state.disabled = false
    }
  }

  execute({ commandState, editorSession }) {
    const { nodeId, disabled } = commandState
    if (!disabled) {
      editorSession.transaction((tx) => {
        let node = tx.get(nodeId)
        node.attr({language: this.config.langauge })
      })
    }
  }
}
