'use strict';

var Command = require('substance/ui/Command');

function SaveCommand() {
  SaveCommand.super.apply(this, arguments);
}

SaveCommand.Prototype = function() {

  this.getCommandState = function(props, context) {
    return {
      disabled: false,
      active: false
    };
  };

  this.execute = function(props, context) {
    return {
      status: null
    };
  };

};

Command.extend(SaveCommand);

module.exports = SaveCommand;
