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
    context.documentSession.remote.save(
      exportHTML(context.doc),
      'html'
    )
    return true
  }

}

export default SaveCommand
