import { platform, DefaultDOMElement, AbstractEditor, Toolbar, EditorSession } from 'substance'
import SheetLinter from './SheetLinter'
import SheetStatusBar from './SheetStatusBar'

export default class SheetEditor extends AbstractEditor {

  constructor(...args) {
    super(...args)

    this.__onResize = this.__onResize.bind(this)
    const sheet = this.getDocument()
    this.linter = new SheetLinter(sheet, this.getEditorSession())
    // _cellEditorDoc is used by cell editors (expression bar, or popover editor on enter)
    this._cellEditorDoc = sheet.newInstance()
    // Just adds one cell, used for text editing
    this._cellEditorDoc._node = this._cellEditorDoc.createElement('cell')
    this._cellEditorSession = new CellEditorSession(this._cellEditorDoc, {
      configurator: this.context.editorSession.configurator,
      // EXPERIMENTAL: trying to setup an editor session using the same CommandManager
      // but working on a different doc
      // NOTE: Disabled this, as it was causing problems with SelectAllCommand
      // commandManager: this.context.editorSession.commandManager
    })
  }

  getChildContext() {
    let editorSession = this.props.editorSession
    return Object.assign({}, super.getChildContext(), {
      issueManager: editorSession.issueManager,
      cellEditorSession: this._cellEditorSession
    })
  }

  getInitialState() {
    return {
      showConsole: false,
      consoleContent: null,
      consoleCell: null
    }
  }

  didMount() {
    // always render a second time to render for the real element dimensions
    this.rerender()
    super.didMount()
    if (platform.inBrowser) {
      DefaultDOMElement.wrap(window).on('resize', this._onResize, this)
    }
    this.linter.start()
    const editorSession = this.props.editorSession
    const issueManager = editorSession.issueManager
    issueManager.on('issue:focus', this._onIssueFocus, this)
  }

  dispose() {
    super.dispose()
    if (platform.inBrowser) {
      DefaultDOMElement.wrap(window).off(this)
    }
    const editorSession = this.props.editorSession
    const issueManager = editorSession.issueManager
    issueManager.off(this)
  }

  render($$) {
    let el = $$('div').addClass('sc-sheet-editor')
    el.append(
      this._renderToolbar($$),
      this._renderContent($$),
      this._renderStatusbar($$)
    )
    return el
  }

  _renderToolbar($$) {
    const configurator = this.getConfigurator()
    return $$(Toolbar, {
      toolPanel: configurator.getToolPanel('toolbar')
    }).ref('toolbar')
  }

  _renderContent($$) {
    let el = $$('div').addClass('se-body')
    el.append(
      this._renderSheet($$)
    )
    el.append(
      this._renderConsole($$)
    )
    return el
  }

  _renderSheet($$) {
    const sheet = this.getDocument()
    const linter = this.linter
    // only rendering the sheet when mounted
    // so that we have real width and height
    if (this.isMounted()) {
      const SheetComponent = this.getComponent('sheet')
      return $$(SheetComponent, {
        sheet, linter
      }).ref('sheet')
    } else {
      return $$('div')
    }
  }

  _renderConsole($$) {
    let el = $$('div').addClass('se-console')
    if (this.state.showConsole) {
      let ConsoleContent = this.getComponent(this.state.consoleContent)
      el.append(
        $$(ConsoleContent, { editor: this, cellId: this.state.consoleCell })
      )
    }
    return el
  }

  _renderStatusbar($$) {
    return $$(SheetStatusBar, {}).ref('sheet-statusbar')
  }

  getLinter() {
    return this.linter
  }

  getIssues() {
    let editorSession = this.props.editorSession
    let issueManager = editorSession.issueManager
    return issueManager.getIssues('linter')
  }

  getWidth() {
    if (this.el) {
      return this.el.getWidth()
    } else {
      return 1000
    }
  }

  getHeight() {
    if (this.el) {
      return this.el.getHeight()
    } else {
      return 750
    }
  }

  setSelectionOnCell(cellId) {
    const sheet = this.getDocument()
    let cell = sheet.get(cellId)
    let row = cell.parentNode
    let colIdx = row._childNodes.indexOf(cell.id)
    let rowIdx = row.parentNode._childNodes.indexOf(row.id)
    let selData = {
      type: 'range',
      anchorRow: rowIdx,
      focusRow: rowIdx,
      anchorCol: colIdx,
      focusCol: colIdx
    }

    this.context.editorSession.setSelection({
      type: 'custom',
      customType: 'sheet',
      data: selData,
      surfaceId: this.refs.sheet.getId()
    })
  }

  setSelectionOnSheet() {
    const sheet = this.getDocument()
    let selData = {
      type: 'range',
      anchorRow: 0,
      focusRow: sheet.getRowCount() - 1,
      anchorCol: 0,
      focusCol: sheet.getColumnCount() - 1
    }

    this.context.editorSession.setSelection({
      type: 'custom',
      customType: 'sheet',
      data: selData,
      surfaceId: this.refs.sheet.getId()
    })
  }

  toggleConsole(consoleContent, consoleCell) {
    if (this.state.showConsole && this.state.consoleContent === consoleContent && consoleCell === undefined) {
      this.setState({
        showConsole: false
      })
    } else {
      this.setState({
        showConsole: true,
        consoleContent,
        consoleCell
      })
    }
  }

  _onIssueFocus(cellId) {
    this.toggleConsole('sheet-issues', cellId)
  }

  _onResize() {
    if (platform.inBrowser) {
      if (!this._rafId) {
        this._rafId = window.requestAnimationFrame(this.__onResize)
      }
    }
  }

  __onResize() {
    this._rafId = null
    this.refs.sheet.resize()
  }

}

class CellEditorSession extends EditorSession {

  /*
    Triggered when a cell editor is focused
  */
  startEditing() {
    if (!this.isEditing) {
      this.isEditing = true
      this.emit('editing:started')
    }
  }

  /*
    Triggered when cell editing is confirmed (e.g. enter is pressed in the cell editor)
  */
  confirmEditing(silent) {
    if (this.isEditing) {
      this.isEditing = false
      if (!silent) this.emit('editing:confirmed')
    }
  }

  /*
    Triggered when cell editing is cancelled (e.g. escape is pressed in the cell editor)
  */
  cancelEditing() {
    if (this.isEditing) {
      this.isEditing = false
      this.emit('editing:cancelled')
    }
  }

  /*
    Get the current value of the cell editor
  */
  getValue() {
    let node = this.getDocument()._node
    return node.getText()
  }

}
