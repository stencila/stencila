'use strict';

var ControllerCommand = require('substance/ui/ControllerCommand');

/**
  Custom save command, which triggers opening a save/commit
  dialog where a commit message can be provided.
*/

function SaveCommand() {
  SaveCommand.super.apply(this, arguments);
}

SaveCommand.Prototype = function() {

  this.getCommandState = function() {
    var doc = this.getDocument();
    return {
      disabled: false, // !doc.__dirty,
      active: false
    };
  },

  this.execute = function() {
    this.getController().saveDocument();
    return {
      status: 'saving-process-started'
    };
  };
};

ControllerCommand.extend(SaveCommand);

SaveCommand.static.name = 'save';

module.exports = SaveCommand;
