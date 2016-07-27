'use strict';

var InlineNodeCommand = require('substance/ui/InlineNodeCommand');
var helpers = require('substance/model/documentHelpers');

function PrintCommand() {
  PrintCommand.super.call(this, {
  	name: 'print',
  	nodeType: 'print'
  });
}

PrintCommand.Prototype = function() {

  /**
   * TODO
   * 
   * Currently, if the tool is selected while on an existing `Print`
   * node then the node gets deleted and this new one inserted.
   * Probably need to override `execute` and if current selection
   * is a `Print` node then toggle it (i.e. make content plain text)
   */

  /**
   * Override of `super.createNodeData`
   * 
   * Used when inserting a new node.
   *
   * @param      {<type>}  tx      The transmit
   * @param      {<type>}  args    The arguments
   * @return     {Object}  { description_of_the_return_value }
   */
  this.createNodeData = function(tx, args) {
  	// Create source from current selection
    var text = helpers.getTextForSelection(
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
