'use strict';

var SurfaceCommand = require('substance-fe0ed/ui/SurfaceCommand');
var helpers = require('substance-fe0ed/model/documentHelpers');
var uuid = require('substance-fe0ed/util/uuid');

function StencilTextInsertCommand() {
  StencilTextInsertCommand.super.apply(this, arguments);
}

StencilTextInsertCommand.Prototype = function() {

  this.getCommandState = function() {
    return {
      disabled: false,
      active: true
    };
  };

  this.execute = function() {
    var state = this.getCommandState();
    if (state.disabled) return;

    var sel = this.getSelection();
    var doc = this.getDocument();
    var text = helpers.getTextForSelection(doc,sel);
    
    var surface = this.getSurface();
    surface.transaction(function(tx, args) {
      tx.create({
        type: 'stencil-text', 
        id: uuid('stencil-text'),
        source: text,

        path: sel.getPath(),
        startOffset: sel.getStartOffset(),
        endOffset: sel.getEndOffset()
      });
    }.bind(this));

    surface.getController().renderDocument();

    return true;
  };
};

SurfaceCommand.extend(StencilTextInsertCommand);

StencilTextInsertCommand.static.name = 'stencil-text-insert';

module.exports = StencilTextInsertCommand;
