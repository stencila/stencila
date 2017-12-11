import {
  Component, TextPropertyEditor, isArrayEqual
} from 'substance'

import { getSyntaxTokens } from '../engine/expressionHelpers'

export default
class CodeEditor extends Component {

  render($$) {
    let el = $$('div').addClass('sc-code-editor')
    // the source code
    const path = this.props.path
    const commands = this.props.commands
    const excludedCommands = this.props.excludedCommands
    const markers = this._getMarkers()
    let content = $$(TextPropertyEditor, {
      name: this.props.name,
      path,
      commands,
      excludedCommands,
      markers,
      handleTab: false,
      withoutBreak: this.props.withoutBreak,
      multiLine: this.props.withoutBreak
    }).ref('contentEditor')
      // EXPERIMENTAL: adding "\n" plus indent of current line
      // .on('enter', this._onEnterKey)
      // EXPERIMENTAL: adding 2 spaces if at begin of line
      .on('tab', this._onTabKey)
      .on('escape', this._onEscapeKey)
    content.addClass('se-content')
    el.append(content)
    return el
  }

  _getMarkers() {
    const path = this.props.path
    return getSyntaxTokens(path, this.props.tokens)
  }

  _onEscapeKey() {
    this.send('escape')
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

  _insertNewLine() {
    const editorSession = this.context.editorSession
    const indent = this._getCurrentIndent() || ''
    editorSession.transaction((tx) => {
      tx.insertText('\n' + indent)
    })
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
