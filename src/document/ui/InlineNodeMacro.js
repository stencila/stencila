import { Editing } from 'substance'
import Macro from './Macro'

class InlineNodeMacro extends Macro {

  performAction (match, props, context) {
    var surface = context.surfaceManager.getSurface(props.selection.surfaceId)
    surface.transaction(function (tx, args) {
      var sel = tx.createSelection(props.path, match.index, match.index + match[0].length)
      // Insert a new node (there is no need to delete the matched text, that is
      // done for us)
      let editing = new Editing()
      editing.insertInlineNode(tx, {
        selection: sel,
        node: this.createNodeData(match)
      })
      if (props.action === 'type') {
        // Move caret to just after the newly inserted node
        return {
          selection: tx.createSelection(props.path, match.index + 1)
        }
      }
    }.bind(this))
  }

}

export default InlineNodeMacro
