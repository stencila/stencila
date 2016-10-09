import Command from 'substance/ui/Command'

/**
 * Command for refreshing a Stencila Document
 *
 * @class      CommitCommand (name)
 */
class CommitCommand extends Command {

  getCommandState (props, context) {
    return {
      disabled: false,
      active: false
    }
  }

  execute (props, context) {
    return {
      status: null
    }
  }

}

export default CommitCommand
