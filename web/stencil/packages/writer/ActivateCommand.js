'use strict';

var ControllerCommand = require('substance/ui/ControllerCommand');

var ActivateCommand = ControllerCommand.extend({
  static: {
    name: 'activate'
  },

  getCommandState: function() {
    var doc = this.getDocument();
    return {
      disabled: false,
      active: true
    };
  },

  execute: function() {
    this.getController().activateDocument();
    return {
      status: 'component-activate-started'
    };
  }
});

module.exports = ActivateCommand;
