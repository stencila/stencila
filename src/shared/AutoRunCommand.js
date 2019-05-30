import { Command } from 'substance'

export default class RunAllCommand extends Command {

  getCommandState({ editorSession }) {
    const doc = editorSession.getDocument()
    const autorun = doc.autorun
    return {
      autoOrManual: autorun ? 'Manual' : 'Auto',
      disabled: false
    }
  }

  execute({ editorSession }) {
    let doc = editorSession.getDocument()
    const autorun = doc.autorun
    doc.autorun = !autorun
    editorSession.setSelection(null)
  }

}
