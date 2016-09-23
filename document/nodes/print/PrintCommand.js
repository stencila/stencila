import InlineNodeCommand from 'substance/ui/InlineNodeCommand'
import documentHelpers from 'substance/model/documentHelpers'

class PrintCommand extends InlineNodeCommand {

  constructor () {
    super({
      name: 'print',
      nodeType: 'print'
    })
  }

  createNodeData (tx, args) {
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

export default PrintCommand
