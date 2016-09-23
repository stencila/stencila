'use strict'

import InlineNodeCommand from 'substance/ui/InlineNodeCommand'
import documentHelpers from 'substance/model/documentHelpers'

function PrintCommand () {
  PrintCommand.super.call(this, {
    name: 'print',
    nodeType: 'print'
  })
}

PrintCommand.Prototype = function () {
  this.createNodeData = function (tx, args) {
    // Create source from current selection
    var text = documentHelpers.getTextForSelection(
      tx.document,
      args.selection
    )
    return {
      type: 'print',
      source: text
    }
  }
}

InlineNodeCommand.extend(PrintCommand)

export default PrintCommand
