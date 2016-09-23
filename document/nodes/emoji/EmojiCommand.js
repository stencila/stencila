import InlineNodeCommand from 'substance/ui/InlineNodeCommand'
import documentHelpers from 'substance/model/documentHelpers'

class EmojiCommand extends InlineNodeCommand {

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
