'use strict';

var Macro = require('./Macro');
var insertInlineNode = require('substance/model/transform/insertInlineNode');

function InlineNodeMacro () {
};

InlineNodeMacro.Prototype = function() {
  
  this.performAction = function(match, props, context) {
    var surface = context.surfaceManager.getSurface(props.selection.surfaceId);
    surface.transaction(function(tx, args) {
      var sel = tx.createSelection(props.path, match.index, match.index + match[0].length);
      // Insert a new node (there is no need to delete the matched text, that is
      // done for us)
      var insert = insertInlineNode(tx, {
        selection: sel,
        node: this.createNodeData(match)
      });
      if (props.action === 'type') {
        // Move caret to just after the newly inserted node
        return {
          selection: tx.createSelection(props.path, match.index + 1)
        };
      }
    }.bind(this));
  }

  /**
   * Create an object with the data for the new node
   * 
   * Should be overidden by derived classes.
   * Analagous to the method with the same name
   * in `substance/ui/InlineNodeCommand`.
   *
   * @param      {<type>}  match   The match
   */
  this.createNodeData = function(match) {
    throw new Error('This method is abstract.');
  };

};

Macro.extend(InlineNodeMacro);

module.exports = InlineNodeMacro;
