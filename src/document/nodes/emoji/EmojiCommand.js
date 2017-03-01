import { InsertInlineNodeCommand, documentHelpers } from 'substance'

class EmojiCommand extends InsertInlineNodeCommand {

  constructor () {
    super({
      name: 'emoji',
      nodeType: 'emoji'
    })
  }

  createNodeData (tx, args) {
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

export default EmojiCommand
