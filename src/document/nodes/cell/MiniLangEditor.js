import {
  Component, TextPropertyEditor, isArrayEqual, parseKeyEvent
} from 'substance'

import { getSyntaxTokens } from '../../expressionUtils'

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
    return expression ? getSyntaxTokens(path, expression) : []
  }

  _onEnterKey(event) {
    const data = event.detail
    const modifiers = parseKeyEvent(data, 'modifiers-only')
    switch(modifiers) {
      case 'ALT': {
        this.send('break')
        break
      }
      case 'CTRL': {
        this.send('execute')
        break
      }
      default:
        //
        this._insertNewLine()
    }
  }

  _insertNewLine() {
    const editorSession = this.context.editorSession
    const indent = this._getCurrentIndent() || ''
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
