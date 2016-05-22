'use strict';

var SurfaceCommand = require('substance/ui/SurfaceCommand');
var uuid = require('substance/util/uuid');

function StencilFigureInsertCommand() {
  StencilFigureInsertCommand.super.apply(this, arguments);
}

StencilFigureInsertCommand.Prototype = function() {

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
        type: 'stencil-figure', 
        id: uuid('stencil-figure'),
        // HACK assumes this is an figure directive for R with certain sizes
        spec: 'r format png size 17x12cm',
        source: '',
        caption: 'Caption'
      };
      return surface.insertNode(tx,args);
    }.bind(this));

    return true;
  };
};

SurfaceCommand.extend(StencilFigureInsertCommand);

StencilFigureInsertCommand.static.name = 'stencil-figure-insert';

module.exports = StencilFigureInsertCommand;
