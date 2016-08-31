'use strict';

var InlineNodeCommand = require('substance/ui/InlineNodeCommand');
var documentHelpers = require('substance/model/documentHelpers');

function MathCommand () {

  MathCommand.super.call(this, {
    name: 'math',
    nodeType: 'math'
  });

}

MathCommand.Prototype = function () {

  this.createNodeData = function (tx, args) {

    // Create math node with source set to current selection
    var text = documentHelpers.getTextForSelection(
      tx.document,
      args.selection
    );
    return {
      type: 'math',
      source: text
    };

  };

};

InlineNodeCommand.extend(MathCommand);

module.exports = MathCommand;
