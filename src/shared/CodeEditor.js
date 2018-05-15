import {
  Component, TextPropertyEditor, isArrayEqual
} from 'substance'

import analyseCode from './analyseCode'
import { getSyntaxTokens } from '../shared/expressionHelpers'

export default class CodeEditor extends Component {

  didMount() {
    super.didMount()

    // this is used to run the code analysis
    this.context.editorSession.onUpdate('document', this._onCodeUpdate, this, {
      path: this.props.path
    })

    this._onCodeUpdate()
  }

  dispose() {
    super.dispose()

    this.context.editorSession.off(this)
  }

  render($$) {
    let el = $$('div').addClass('sc-code-editor')
    // the source code
    const path = this.props.path
    const commands = this.props.commands
    const excludedCommands = this.props.excludedCommands
    let content = $$(TextPropertyEditor, {
      // TextPropertyEditor props
      name: this.props.name,
      path,
      multiLine: this.props.multiline,
      // Surface props
      commands,
      excludedCommands,
      handleEnter: false,
      handleTab: false
    }).ref('contentEditor')
    if (this.props.multiline) {
      content.on('enter', this._onEnterKey)
      content.on('tab', this._onTabKey)
    }
    content.addClass('se-content')
    el.append(content)
    return el
  }

  getSurfaceId() {
    return this.refs.contentEditor.getId()
  }

  _onCodeUpdate() {
    let code = this._getCode()
    let shouldAnalyse = true
    // TODO: how can we generalize this?
    // in spreadsheet cells there must be a leading '=' to be
    // considered as expression
    if (this.props.mode === 'cell') {
      shouldAnalyse = Boolean(/^\s*=/.exec(code))
    }
    // tokens for syntax-highlighting
    let tokens = []
    // symbols such as 'var', 'cell', or 'range'
    let symbols = []
    // detected complex nodes, such as function calls
    let nodes = []
    if (shouldAnalyse) {
      ({tokens, symbols, nodes} = analyseCode(code, this.props.language))
    }
    this._setMarkers(tokens)
    // TODO: rethink - if there was a State API how would we do this?
    // want to share code analysis e.g. with Commands
    this._extendState({ tokens, symbols, nodes })
  }

  _getCode() {
    const path = this.props.path
    return this.context.editorSession.getDocument().get(path)
  }

  _setMarkers(tokens) {
    const path = this.props.path
    const markersManager = this.context.editorSession.markersManager
    // TODO: renamve this helper to `getMarkersForTokens`
    let markers = getSyntaxTokens(path, tokens)
    markersManager.setMarkers(`code-analysis@${path.join('.')}`, markers)
  }

  _extendState(values) {
    // TODO: do we really want this?
    let state = this._getState()
    Object.assign(state, values)
  }

  _getState() {
    // TODO: this should be general, not tied to Stencila Cells
    const path = this.props.path
    const nodeId = path[0]
    const node = this.context.editorSession.getDocument().get(nodeId)
    if (!node.state) {
      node.state = {}
    }
    return node.state
  }

  _onTabKey(e) {
    e.stopPropagation()
    const editorSession = this.context.editorSession
    const head = this._getCurrentLineHead()
    // console.log('head', head)
    if (/^\s*$/.exec(head)) {
      editorSession.transaction((tx) => {
        tx.insertText('  ')
      })
    }
  }

  // only used if multiline=true
  _onEnterKey(e) {
    e.stopPropagation()
    this._insertNewLine()
  }

  _insertNewLine() {
    const editorSession = this.context.editorSession
    const indent = this._getCurrentIndent()
    editorSession.transaction((tx) => {
      tx.insertText('\n' + indent)
    })
  }

  _getCurrentIndent() {
    const line = this._getCurrentLineHead()
    const match = /^(\s+)/.exec(line)
    if (match) {
      return match[1]
    } else {
      return ''
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
    const lastNL = head.lastIndexOf('\n')
    return head.slice(lastNL+1)
  }
}
