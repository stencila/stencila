'use strict';

var ControllerCommand = require('substance/ui/ControllerCommand');

/**
 * Commit the sheet
 */
function SaveCommand() {
  SaveCommand.super.apply(this, arguments);
}

SaveCommand.Prototype = function() {

  this.getCommandState = function() {
    return {
      disabled: false,
      active: false
    };
  },

  this.execute = function() {
    var engine = this.controller.props.engine;
    engine.save();
    return {
      status: 'save-requested'
    };
  };
};

ControllerCommand.extend(SaveCommand);
SaveCommand.static.name = 'save';

module.exports = SaveCommand;
