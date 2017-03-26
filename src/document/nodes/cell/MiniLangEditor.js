import {
  Component, TextPropertyEditor, isArrayEqual
} from 'substance'

export default
class MiniLangEditor extends Component {

  render($$) {
    let el = $$('div').addClass('sc-mini-lang-editor')
    // the source code
    const path = this.props.path
    const commands = this.props.commands
    const markers = this._getMarkers()
    let content = $$(TextPropertyEditor, {
      path,
      commands,
      markers,
      handleTab: false
    }).ref('contentEditor')
      // NOTE: disable these if these are causing troubles
      // EXPERIMENTAL: adding "\n" plus indent of current line
      .on('enter', this._onEnterKey)
      // EXPERIMENTAL: adding 2 spaces if at begin of line
      .on('tab', this._onTabKey)
    content.addClass('se-content')
    el.append(content)
    return el
  }

  _getMarkers() {
    const expression = this.props.expression
    const path = this.props.path
    if (expression) {
      return expression.tokens.map((t) => {
        return {
          type: 'code-highlight',
          name: t.type,
          start: { path, offset: t.start },
          end: { path, offset: t.end },
          on() {},
          off() {}
        }
      })
    } else {
      return []
    }
  }

  _onEnterKey() {
    // find the indent of the current line
    let indent = this._getCurrentIndent() || ''
    const editorSession = this.context.editorSession
    editorSession.transaction((tx) => {
      tx.insertText('\n' + indent)
    })
  }

  _onTabKey() {
    const editorSession = this.context.editorSession
    const head = this._getCurrentLineHead()
    // console.log('head', head)
    if (/^\s*$/.exec(head)) {
      editorSession.transaction((tx) => {
        tx.insertText('  ')
      })
    }
  }

  _getCurrentIndent() {
    const line = this._getCurrentLineHead()
    const match = /^(\s+)/.exec(line)
    if (match) {
      return match[1]
    }
  }

  _getCurrentLineHead() {
    const editorSession = this.context.editorSession
    const doc = editorSession.getDocument()
    const sel = editorSession.getSelection()
    if (!sel || !sel.isPropertySelection() || !isArrayEqual(sel.path, this.props.path)) {
      return
    }
    const offset = sel.start.offset
    const exprStr = doc.get(this.props.path)
    const head = exprStr.slice(0, offset)
    const lastNL = Math.max(0, head.lastIndexOf('\n'))
    return head.slice(lastNL+1)
  }
}
