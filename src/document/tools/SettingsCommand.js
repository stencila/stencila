import { Command } from 'substance'

class SettingsCommand extends Command {

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

export default SettingsCommand
