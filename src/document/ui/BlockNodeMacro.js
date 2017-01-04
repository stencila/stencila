import Macro from './Macro'
import Editing from 'substance/model/Editing'

class BlockNodeMacro extends Macro {

  get appliesTo () {
    return ['paragraph']
  }

  performAction (match, props, context) {
    var surface = context.surfaceManager.getSurface(props.selection.surfaceId)
    surface.transaction(function (tx, args) {
      // Create the new node
      var newNode = tx.create(
        this.createNodeData(match)
      )

      // Hide the old node, show the new node
      var container = tx.get(args.containerId)
      var pos = container.getPosition(props.node.id)
      if (pos >= 0) {
        container.hide(props.node.id)
        container.show(newNode.id, pos)
      }

      // Delete the old node
      let editing = new Editing()
      editing.deleteNode(tx, props.node.id, args.containerId)

      // Set the selection
      var path
      if (newNode.isText()) path = newNode.getTextPath()
      else path = [newNode.id]
      args.selection = tx.createSelection(path, 0)

      return args
    }.bind(this))
  }

}

export default BlockNodeMacro
