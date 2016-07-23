'use strict';

var SurfaceCommand = require('substance-fe0ed/ui/SurfaceCommand');
var createTable = require('./createTable');

// TODO: this should go into substance/packages/table/
function InsertTableCommand() {
  InsertTableCommand.super.apply(this, arguments);
}

InsertTableCommand.Prototype = function() {

  this.getCommandState = function() {
    var sel = this.getSelection();
    var newState = {
      disabled: true,
      active: false
    };
    if (sel && !sel.isNull() && sel.isPropertySelection()) {
      newState.disabled = false;
    }
    return newState;
  };

  this.execute = function() {
    var state = this.getCommandState();
    // Return if command is disabled
    if (state.disabled) return;

    var surface = this.getSurface();
    surface.transaction(function(tx, args) {
      args.rows = 10;
      args.cols = 10;
      var out = createTable(tx, args);
      args.node = out.table;
      // Note: returning the result which will contain an updated selection
      return surface.insertNode(tx, args);
    }.bind(this));
  };
};

SurfaceCommand.extend(InsertTableCommand);

InsertTableCommand.static.name = 'insertTable';

module.exports = InsertTableCommand;