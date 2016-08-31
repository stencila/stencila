'use strict';

var Macro = require('./Macro');
var deleteNode = require('substance/model/transform/deleteNode');

function BlockNodeMacro () {
};

BlockNodeMacro.Prototype = function () {

  this.appliesTo = ['paragraph'];

  this.performAction = function (match, props, context) {

    var surface = context.surfaceManager.getSurface(props.selection.surfaceId);
    surface.transaction(function (tx, args) {

      // Create the new node
      var newNode = tx.create(
        this.createNodeData(match)
      );

      // Hide the old node, show the new node
      var container = tx.get(args.containerId);
      var pos = container.getPosition(props.node.id);
      if (pos >= 0) {

        container.hide(props.node.id);
        container.show(newNode.id, pos);

      }

      // Delete the old node
      deleteNode(tx, { nodeId: props.node.id });

      // Set the selection
      var path;
      if (newNode.isText()) path = newNode.getTextPath();
      else path = [newNode.id];
      args.selection = tx.createSelection(path, 0);

      return args;

    }.bind(this));

  };

};

Macro.extend(BlockNodeMacro);

module.exports = BlockNodeMacro;
