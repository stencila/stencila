'use strict'

import Command from 'substance/ui/Command'

/**
 * Command for refreshing a Stencila Document
 *
 * @class      CommitCommand (name)
 */
function CommitCommand () {
  CommitCommand.super.apply(this, arguments)
}

CommitCommand.Prototype = function () {
  this.getCommandState = function (props, context) {
    return {
      disabled: false,
      active: false
    }
  }

  this.execute = function (props, context) {
    return {
      status: null
    }
  }
}

Command.extend(CommitCommand)

export default CommitCommand
