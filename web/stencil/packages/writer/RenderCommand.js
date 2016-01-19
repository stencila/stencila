'use strict';

var ControllerCommand = require('substance/ui/ControllerCommand');

function RenderCommand() {
  RenderCommand.super.apply(this, arguments);
}

RenderCommand.Prototype = function() {
  this.getCommandState = function() {
    var doc = this.getDocument();
    return {
      disabled: doc.__isRendering,
      active: false
    };
  };

  this.execute = function() {
    this.getController().renderDocument();
    return {
      status: 'render-process-started'
    };
  };

};


ControllerCommand.extend(RenderCommand);

RenderCommand.static.name = 'render';

module.exports = RenderCommand;
