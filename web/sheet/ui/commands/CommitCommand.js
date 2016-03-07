'use strict';

var ControllerCommand = require('substance/ui/ControllerCommand');

/**
  Custom save command, which triggers opening a save/commit
  dialog where a commit message can be provided.
*/

function CommitCommand() {
  CommitCommand.super.apply(this, arguments);
}

CommitCommand.Prototype = function() {

  this.getCommandState = function() {
    var doc = this.getDocument();
    return {
      disabled: false, // !doc.__dirty,
      active: false
    };
  },

  this.execute = function() {
    return {
      status: 'save-requested'
    };
  };
};

ControllerCommand.extend(CommitCommand);

CommitCommand.static.name = 'save';

module.exports = CommitCommand;
