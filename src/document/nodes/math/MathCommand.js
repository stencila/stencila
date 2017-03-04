import { InsertInlineNodeCommand, documentHelpers } from 'substance'

class MathCommand extends InsertInlineNodeCommand {

  // constructor () {
  //   super({
  //     name: 'math',
  //     nodeType: 'math'
  //   })
  // }

  createNodeData (tx, args) {
    // Create math node with source set to current selection
    var text = documentHelpers.getTextForSelection(
      tx.document,
      args.selection
    )
    return {
      type: 'math',
      source: text
    }
  }

}

export default MathCommand
