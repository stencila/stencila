import { Command } from 'substance'

export default class FunctionUsageCommand extends Command {
  getCommandState() {
    let newState = {
      disabled: false,
      functionName: 'sum',
      paramIndex: 0
    }
    return newState
  }

  execute(params) { } // eslint-disable-line
}
