import { Command } from 'substance'

export default class RunAllCommand extends Command {

  getCommandState({ editorSession }) {
    const doc = editorSession.getDocument()
    const autorun = doc.autorun

    return {
      disabled: autorun
    }
  }
}
