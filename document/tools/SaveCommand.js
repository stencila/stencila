import Command from 'substance/ui/Command'

class SaveCommand extends Command {

  getCommandState (props, context) {
    return {
      disabled: false,
      active: false
    }
  }

  execute (props, context) {
    context.doc.save()
    return true
  }

}

export default SaveCommand
