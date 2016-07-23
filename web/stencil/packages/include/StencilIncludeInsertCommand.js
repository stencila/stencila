'use strict';

var SurfaceCommand = require('substance-fe0ed/ui/SurfaceCommand');
var helpers = require('substance-fe0ed/model/documentHelpers');
var uuid = require('substance-fe0ed/util/uuid');

function StencilIncludeInsertCommand() {
  StencilIncludeInsertCommand.super.apply(this, arguments);
}

StencilIncludeInsertCommand.Prototype = function() {

  this.getCommandState = function() {
    return {
      disabled: false,
      active: true
    };
  };

  this.execute = function() {
    var state = this.getCommandState();
    if (state.disabled) return;

    var selection = this.getSelection();
    var doc = this.getDocument();
    var text = helpers.getTextForSelection(doc,selection);
    
    var surface = this.getSurface();
    surface.transaction(function(tx, args) {
      args.node = {
        type: 'stencil-include', 
        id: uuid('stencil-include'),
        source: text
      };
      return surface.insertNode(tx,args);
    }.bind(this));

    surface.getController().renderDocument();

    return true;
  };
};

SurfaceCommand.extend(StencilIncludeInsertCommand);

StencilIncludeInsertCommand.static.name = 'stencil-include-insert';

module.exports = StencilIncludeInsertCommand;
