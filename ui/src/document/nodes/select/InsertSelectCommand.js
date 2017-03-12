import { InsertInlineNodeCommand, documentHelpers } from 'substance'

class InsertSelectCommand extends InsertInlineNodeCommand {

  createNodeData (tx, args) {
    var text = documentHelpers.getTextForSelection(
      tx.getDocument(),
      args.selection
    )

    return {
      type: 'select',
      name: '',
      options: [
        { text: text, value: text }
      ],
      selectedIndex: 0
    }
  }
}

export default InsertSelectCommand
