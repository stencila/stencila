'use strict';

var oo = require('substance/util/oo');
var insertInlineNode = require('substance/model/transform/insertInlineNode');
var deleteSelection = require('substance/model/transform/deleteSelection');

function InlineNodeMacro () {
};

InlineNodeMacro.Prototype = function() {

  this.appliesTo = ['paragraph'];
  
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

  this.execute = function(props, context) {
    if (!this.regex) {
      throw new Error('Must define `this.regex` for InlineNodeMacro class');
    }

    if (this.appliesTo.indexOf(props.node.type) === -1) {
      return false;
    }

    var match = this.regex.exec(props.text);
    if (match) {
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
      return true;
    }
  }

};

oo.initClass(InlineNodeMacro);

module.exports = InlineNodeMacro;
