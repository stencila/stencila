import { Command } from 'substance'

export class SetLanguageCommand extends Command {

  getCommandState({ selection, editorSession }) {
    let doc = editorSession.getDocument()
    if (selection.isNodeSelection()) {
      let nodeId = selection.getNodeId()
      let node = doc.get(nodeId)
      if (node.type === 'cell') {
        let language = node.find('source-code').attr('language')
        return {
          cellId: node.id,
          newLanguage: this.config.language,
          disabled: false,
          active: this.config.language === language
        }
      }
    }
    return { disabled: true }
  }

  execute({ editorSession, commandState }) {
    let { cellId, newLanguage, disabled } = commandState
    if (!disabled) {
      editorSession.transaction((tx) => {
        let cell = tx.get(cellId)
        let sourceCode = cell.find('source-code')
        sourceCode.attr({ language: newLanguage })
      })
    }
  }
}
