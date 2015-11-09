'use strict';

var ControllerCommand = require('substance/ui/ControllerCommand');

var RenderCommand = ControllerCommand.extend({
  static: {
    name: 'render'
  },

  getCommandState: function() {
    var doc = this.getDocument();
    return {
      disabled: doc.__isRendering,
      active: false
    };
  },

  execute: function() {
    this.getController().renderDocument();
    return {
      status: 'render-process-started'
    };
  }
});

module.exports = RenderCommand;
