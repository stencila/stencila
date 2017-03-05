import { InsertInlineNodeCommand, documentHelpers } from 'substance'

class MathCommand extends InsertInlineNodeCommand {

  createNodeData (tx, args) {
    // Create math node with source set to current selection
    var text = documentHelpers.getTextForSelection(
      tx.getDocument(),
      args.selection
    )
    return {
      type: 'math',
      source: text
    }
  }

}

export default MathCommand
