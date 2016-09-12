'use strict';

var InlineNodeCommand = require('substance/ui/InlineNodeCommand');
var documentHelpers = require('substance/model/documentHelpers');

function EmojiCommand () {
  EmojiCommand.super.call(this, {
    name: 'emoji',
    nodeType: 'emoji'
  });
}

EmojiCommand.Prototype = function () {
  this.createNodeData = function (tx, args) {
    // Create emoji node with name set to current selection
    var text = documentHelpers.getTextForSelection(
      tx.document,
      args.selection
    );
    return {
      type: 'emoji',
      name: text
    };
  };
};

InlineNodeCommand.extend(EmojiCommand);

module.exports = EmojiCommand;
