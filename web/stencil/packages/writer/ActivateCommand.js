'use strict';

var ControllerCommand = require('substance-fe0ed/ui/ControllerCommand');

function ActivateCommand() {
  ActivateCommand.super.apply(this, arguments);
}

ActivateCommand.Prototype = function() {
  this.getCommandState = function() {
    var doc = this.getDocument();
    return {
      disabled: false,
      active: true
    };
  };

  this.execute = function() {
    this.getController().activateDocument();
    return {
      status: 'component-activate-started'
    };
  };
};

ControllerCommand.extend(ActivateCommand);

ActivateCommand.static.name = 'activate';

module.exports = ActivateCommand;
