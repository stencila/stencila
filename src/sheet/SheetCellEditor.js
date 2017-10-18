import { AbstractEditor, TextPropertyEditor } from 'substance'

export default class SheetCellEditor extends AbstractEditor {

  _initialize(props) {
    super._initialize(props)
    this.node = props.editorSession.getDocument()._node
    // Initialize pseudo node when node is given on construction
    if (props.node) {
      this.node.setText(props.node.getText())
    }
  }

  willReceiveProps(props) {
    const node = props.node
    if (node) {
      this.node.setText(node.getText())
    }
  }

  render($$) {
    let el = $$('div').addClass('sc-sheet-cell-editor')
    el.append(
      $$(TextPropertyEditor, {
        name: this.props.name,
        path: this.node.getPath()
      }).ref('editor')
        .on('focus', this._onFocus)
        .on('contextmenu', this._onContextMenu)
    )
    el.on('mousedown', this._onMousedown)
      .on('enter', this._onEnter)
      .on('escape', this._onEscape)
    return el
  }

  _onEnter() {
    this.context.cellEditorSession.confirmEditing()
  }

  _onEscape() {
    this.context.cellEditorSession.cancelEditing()
  }

  _onFocus() {
    this.context.cellEditorSession.startEditing()
  }

  focus() {
    let cellStr = this.node.getText()
    this.editorSession.setSelection({
      type: 'property',
      path: this.node.getPath(),
      startOffset: cellStr.length,
      surfaceId: this.refs.editor.getSurfaceId()
    })
  }

  _onMousedown(e) {
    e.stopPropagation()
  }

  _onContextMenu(e) {
    e.preventDefault()
  }

}
