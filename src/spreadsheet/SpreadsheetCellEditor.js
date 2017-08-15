import { EditorSession, AbstractEditor, TextPropertyEditor } from 'substance'

export default class SpreadsheetCellEditor extends AbstractEditor {

  _initialize(props) {
    let doc = props.sheet.newInstance()
    let node = doc.createElement('cell')
    let editorSession = new EditorSession(doc, {
      // EXPERIMENTAL: trying to setup an editor session using the same CommandManager
      // but working on a different doc
      configurator: this.context.editorSession.configurator,
      commandManager: this.context.commandManager
    })

    super._initialize({ editorSession })
    this.node = node
  }

  willReceiveProps(props) {
    const node = props.node
    if (node) {
      this.node.setText(node.getText())
    }
  }

  render($$) {
    let el = $$('div').addClass('sc-spreadsheet-cell-editor')
    el.append(
      $$(TextPropertyEditor, {
        path: this.node.getTextPath()
      }).ref('editor')
        .on('contextmenu', this._onContextMenu)
    )
    el.on('mousedown', this._onMousedown)

    return el
  }

  getValue() {
    return this.node.getText()
  }

  focus() {
    let cellStr = this.node.getText()
    this.editorSession.setSelection({
      type: 'property',
      path: this.node.getTextPath(),
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