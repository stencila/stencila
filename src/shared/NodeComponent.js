import { Component } from 'substance'

/*
  A component used to render a DocumentNode.

  It rerenders on updates off the node automatically
 */
export default
class NodeComponent extends Component {

  didMount() {
    _startListening(this, this.props.node)
  }

  dispose() {
    _stopListening(this, this.props.node)
  }

  willReceiveProps(newProps) {
    if (newProps.node !== this.props.node) {
      _stopListening(this, this.props.node)
      _startListening(this, newProps.node)
    }
  }

  getNode() {
    return this.props.node
  }

  _onNodeChanged() {
    this.rerender()
  }
}

function _startListening(self, node) {
  if (node) {
    const editorSession = self.context.editorSession
    editorSession.on('render', self._onNodeChanged, self, {
      // only get updated on document changes
      resource: 'document',
      // and only if the node is affected
      path: [node.id]
    })
  }
}

function _stopListening(self, node) {
  if (node) {
    const editorSession = self.context.editorSession
    editorSession.off('render', self._onNodeChanged, self)
  }
}