import { InsertNodeCommand } from 'substance'

class InsertCellCommand extends InsertNodeCommand {

  createNodeData() {
    return { type: 'cell' }
  }

  getCommandState(params) {
    let commandState = super.getCommandState(params)
    return commandState
  }

  setSelection(tx, node) {
    const containerId = tx.selection.containerId
    tx.selection = {
      type: 'property',
      path: [node.id, 'expression'],
      startOffset: 0,
      // HACK: hand-crafting a surface id
      // this should be easier to do
      surfaceId: [containerId, node.id, `${node.id}.expression`].join('/')
    }
  }

}

export default InsertCellCommand
