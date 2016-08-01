'use strict';

var InlineNodeCommand = require('substance/ui/InlineNodeCommand');
var documentHelpers = require('substance/model/documentHelpers');


function PrintCommand() {
  PrintCommand.super.call(this, {
  	name: 'print',
  	nodeType: 'print'
  });
}

PrintCommand.Prototype = function() {

  this.getCommandState = function(props, context) {
    // Active if a print node is selected, disabled if no node and no
    // text selected.
    var annos = props.selectionState.getAnnotationsForType(this.getNodeType());
    var text = documentHelpers.getTextForSelection(
      props.selectionState.document,
      props.selectionState.selection
    );
    var textSelected = false;
    if (text) {
      if (text.length > 0) textSelected = true;
    }
    return {
      disabled: (annos.length == 0) && !textSelected,
      active: annos.length > 0
    };
  };

  this.createNodeData = function(tx, args) {
  	// Create source from current selection
    var text = documentHelpers.getTextForSelection(
    	tx.document,
    	args.selection
    );
    return {
      type: 'print',
      source: text,
      status: 'uneval',
      content: '?'
    };
  };

};

InlineNodeCommand.extend(PrintCommand);

module.exports = PrintCommand;
