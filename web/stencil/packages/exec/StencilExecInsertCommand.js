'use strict';

var SurfaceCommand = require('substance-fe0ed/ui/SurfaceCommand');
var uuid = require('substance-fe0ed/util/uuid');

function StencilExecInsertCommand() {
  StencilExecInsertCommand.super.apply(this, arguments);
}

StencilExecInsertCommand.Prototype = function() {

  this.getCommandState = function() {
    return {
      disabled: false,
      active: true
    };
  };

  this.execute = function() {
    var state = this.getCommandState();
    if (state.disabled) return;

    var surface = this.getSurface();
    surface.transaction(function(tx, args) {
      args.node = {
        type: 'stencil-exec', 
        id: uuid('stencil-exec'),
        // HACK assumes this is an exec directive for R
        spec: 'r'
      };
      return surface.insertNode(tx,args);
    }.bind(this));

    return true;
  };
};

SurfaceCommand.extend(StencilExecInsertCommand);

StencilExecInsertCommand.static.name = 'stencil-exec-insert';

module.exports = StencilExecInsertCommand;
