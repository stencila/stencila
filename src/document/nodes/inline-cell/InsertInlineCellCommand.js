import { InsertInlineNodeCommand, documentHelpers } from 'substance'

class InsertInlineCellCommand extends InsertInlineNodeCommand {

  createNodeData (tx, args) {
    var text = documentHelpers.getTextForSelection(
      tx.getDocument(),
      args.selection
    )

    return {
      type: 'inline-cell',
      expression: text,
      output: '???'
    }
  }

}

export default InsertInlineCellCommand
