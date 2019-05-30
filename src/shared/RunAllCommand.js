import { Command } from 'substance'

export default class RunAllCommand extends Command {

  getCommandState({ editorSession }) {
    const doc = editorSession.getDocument()
    const autorun = doc.autorun
    return {
      disabled: autorun
    }
  }

  execute(params, context) {
    const editorSession = params.editorSession
    const engine = context.host.engine
    const doc = editorSession.getDocument()
    engine._allowRunningAllCellsOfDocument(doc.id)
  }
}
