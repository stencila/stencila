import { ToolDropdown } from 'substance'

export default class SwitchTypeDropdown extends ToolDropdown {

  _getToggleName(commandStates) {
    let toggleName = super._getToggleName(commandStates)
    // EXPERIMENTAL: trying to bring back the old way of using the
    // switch text type dropdown label as place to show th current context
    // TODO: we should add a helper in substance for retrieving the 'context' of a selection
    if (!toggleName) {
      const editorSession = this.context.editorSession
      const doc = editorSession.getDocument()
      const sel = editorSession.getSelection()
      if (sel && !sel.isNull()) {
        // Try to get the node type from the property selection
        if (sel.isPropertySelection()) {
          let nodeId = sel.start.getNodeId()
          const node = doc.get(nodeId)
          return _nodeLabel(node)
        }
        // HACK: trying to get a label when cursor is inside of external source editor
        else if (sel.isCustomSelection() && sel.surfaceId) {
          const surface = editorSession.getSurface(sel.surfaceId)
          if (surface) {
            const isolatedNodeComp = surface.context.isolatedNodeComponent
            if (isolatedNodeComp) {
              return _nodeLabel(isolatedNodeComp.props.node)
            }
          }
        } else if (sel.isNodeSelection()) {
          let nodeId = sel.getNodeId()
          const node = doc.get(nodeId)
          return _nodeLabel(node)
        }
      }
    }
    return toggleName
  }

}

function _nodeLabel(node) {
  if (node) {
    return node.type
  }
}