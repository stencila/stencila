import { Component } from 'substance'
import CodeEditorComponent from '../../ui/CodeEditorComponent'

export default class CodeblockComponent extends Component {

  constructor(...args) {
    super(...args)

    this.handleActions({
      // triggered by CodeEditorComponent
      'break': this._onBreak
    })
  }


  didMount() {
    const node = this.props.node
    const editorSession = this.context.editorSession
    editorSession.on('render', this._onNodeChanged, this, {
      resource: 'document',
      path: [node.id]
    })
  }

  dispose() {
    const editorSession = this.context.editorSession
    editorSession.off(this)
  }

  render ($$) {
    const node = this.props.node
    let el = $$('div').addClass('sc-codeblock')

    let editor = $$(CodeEditorComponent, {
      path: [node.id, 'source'],
      language: node.language,
      readOnly: this.props.disabled
    }).addClass('se-editor').ref('editor')
      .on('escape', this._onEscapeFromCodeEditor)

    el.append(editor)

    return el
  }

  _onNodeChanged() {
    this.rerender()
  }

  _onEscapeFromCodeEditor(event) {
    event.stopPropagation()
    this.send('escape')
  }

  _onBreak() {
    this.context.editorSession.transaction((tx) => {
      tx.selection = this._afterNode()
      tx.insertBlockNode({
        type: 'paragraph'
      })
    })
  }

  _afterNode() {
    // TODO: not too happy about how difficult it is
    // to set the selection
    const node = this.props.node
    const isolatedNode = this.context.isolatedNodeComponent
    const parentSurface = isolatedNode.getParentSurface()
    return {
      type: 'node',
      nodeId: node.id,
      mode: 'after',
      containerId: parentSurface.getContainerId(),
      surfaceId: parentSurface.id
    }
  }
}

CodeblockComponent.fullWidth = true
