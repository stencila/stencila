'use strict'

import InlineNodeCommand from 'substance/ui/InlineNodeCommand'
import documentHelpers from 'substance/model/documentHelpers'

function EmojiCommand () {
  EmojiCommand.super.call(this, {
    name: 'emoji',
    nodeType: 'emoji'
  })
}

EmojiCommand.Prototype = function () {
  this.createNodeData = function (tx, args) {
    // Create emoji node with name set to current selection
    var text = documentHelpers.getTextForSelection(
      tx.document,
      args.selection
    )
    return {
      type: 'emoji',
      name: text
    }
  }
}

InlineNodeCommand.extend(EmojiCommand)

export default EmojiCommand
