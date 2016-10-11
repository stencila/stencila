import Command from 'substance/ui/Command'

import {exportHTML} from '../documentConversion'

class SaveCommand extends Command {

  getCommandState (props, context) {
    return {
      disabled: false,
      active: false
    }
  }

  execute (props, context) {
    let html = exportHTML(context.doc)
    context.documentSession.remote.save(html, 'html')
    return true
  }

}

export default SaveCommand
