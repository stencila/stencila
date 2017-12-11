import {
  platform, DefaultDOMElement, AbstractEditor,
  Toolbar, EditorSession, DOMSelection
} from 'substance'
import SheetContextSection from './SheetContextSection'
import SheetStatusBar from './SheetStatusBar'
import FormulaBar from './FormulaBar'

export default class SheetEditor extends AbstractEditor {

  constructor(...args) {
    super(...args)

    // an editor session for the overlay cell editor
    // and the expression bar
    this._formulaEditorContext = this._createFormulaEditorContext()
    this._isEditing = false
  }

  getChildContext() {
    let editorSession = this.props.editorSession
    return Object.assign({}, super.getChildContext(), {
      configurator: editorSession.getConfigurator(),
      host: this.props.host,
      issueManager: editorSession.getManager('issue-manager'),
      // TODO: get rid of this
      cellEditorSession: this._cellEditorSession
    })
  }

  getInitialState() {
    return {
      showContext: false,
      contextId: null,
      cellId: null
    }
  }

  didMount() {
    super.didMount()

    const editorSession = this.props.editorSession
    if (platform.inBrowser) {
      DefaultDOMElement.wrap(window).on('resize', this._onResize, this)
    }
    editorSession.onUpdate('selection', this._onSelectionChange, this)

    this._formulaEditorContext.editorSession.onUpdate('selection', this._onCellEditorSelectionChange, this)
    // TODO: is it really necessary to have this in the first line?
    // always render a second time to render for the real element dimensions
    // because the table with fixed layout needs exact dimensions
    this.rerender()
  }

  dispose() {
    super.dispose()

    const editorSession = this.props.editorSession
    if (platform.inBrowser) {
      DefaultDOMElement.wrap(window).off(this)
    }
    editorSession.off(this)
    this._formulaEditorContext.editorSession.off(this)
  }

  render($$) {
    let el = $$('div').addClass('sc-sheet-editor')
    el.append(
      $$('div').addClass('se-main-section').append(
        this._renderToolpane($$),
        this._renderContent($$),
        this._renderStatusbar($$)
      )
    )

    if(this.state.showContext) {
      el.append(
        $$(SheetContextSection, {
          contextId: this.state.contextId,
          cellId: this.state.cellId
        }).ref('context')
      )
    }

    return el
  }

  _renderToolpane($$) {
    const configurator = this.getConfigurator()
    let el = $$('div').addClass('se-tool-pane')
    el.append(
      $$(Toolbar, {
        toolPanel: configurator.getToolPanel('toolbar')
      }).ref('toolbar'),
      $$(FormulaBar, {
        node: this._formulaEditorContext.node,
        context: this._formulaEditorContext
      })
    )
    return el
  }

  _renderContent($$) {
    let el = $$('div').addClass('se-body')
    el.append(
      this._renderSheet($$)
    )
    return el
  }

  _renderSheet($$) {
    const sheet = this.getDocument()
    const formulaEditorContext = this._formulaEditorContext
    // only rendering the sheet when mounted
    // so that we have real width and height
    if (this.isMounted()) {
      const SheetComponent = this.getComponent('sheet')
      return $$(SheetComponent, {
        sheet,
        formulaEditorContext
      }).ref('sheet')
    } else {
      return $$('div')
    }
  }

  _renderStatusbar($$) {
    return $$(SheetStatusBar, {}).ref('sheet-statusbar')
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

    this.props.editorSession.setSelection({
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

    this.props.editorSession.setSelection({
      type: 'custom',
      customType: 'sheet',
      data: selData,
      surfaceId: this.refs.sheet.getId()
    })
  }

  toggleContext(contextId, cellId) {
    if(cellId === null && !this.state.showContext) return
    if (this.state.showContext && this.state.contextId === contextId && cellId === undefined) {
      this.setState({
        showContext: false
      })
    } else {
      this.setState({
        showContext: true,
        contextId,
        cellId
      })
    }
  }

  // a context propagated by FormulaBar and FormulaEditor
  _createFormulaEditorContext() {
    const configurator = this.props.editorSession.configurator
    // a document with only one node used by cell editors
    // i.e. expression bar, or popover editor on enter
    let cellEditorDoc = this.getDocument().newInstance()
    // TODO: use an id instead of storing the node on the doc
    let node = cellEditorDoc.createElement('cell')
    let editorSession = new EditorSession(cellEditorDoc, {
      configurator: configurator
    })
    const self = this
    // HACK: creating inline an object which DOMSelection needs to work
    // TODO: either use a helper to create the DOMSelection
    // or change DOMSelection's ctor to be better usable
    let domSelection = new DOMSelection({
      getDocument() { return cellEditorDoc },
      getSurfaceManager() { return editorSession.surfaceManager },
      getElement() { return self.getElement() }
    })
    return {
      editorSession,
      domSelection,
      node,
      markersManager: editorSession.markersManager,
      surfaceManager: editorSession.surfaceManager
    }
  }

  _onResize() {
    if (platform.inBrowser) {
      if (!this._rafId) {
        this._rafId = window.requestAnimationFrame(() => {
          this._rafId = null
          this.refs.sheet.resize()
        })
      }
    }
  }

  _getSheetSelection() {
    return this.getEditorSession().getSelection().data || {}
  }

  _onSelectionChange() {
    let formulaEditorSession = this._formulaEditorContext.editorSession
    this._setFormula()
    if (this._isEditing) {
      this._isEditing = false
      this.refs.sheet.hideFormulaEditor()
      formulaEditorSession.setSelection(null)
      console.log('_isEditing FALSE')
    }
    
  }

  _setFormula() {
    let sel = this._getSheetSelection()
    let cell = this.getDocument().getCell(sel.anchorRow, sel.anchorCol)
    let context = this._formulaEditorContext
    let node = context.node
    let formulaEditorSession = this._formulaEditorContext.editorSession
    formulaEditorSession.transaction(tx => {
      tx.set(node.getPath(), cell.textContent)
    })
  }

  _onCellEditorSelectionChange(sel) {
    let sheetSel = this._getSheetSelection()
    let formulaEditorSession = this._formulaEditorContext.editorSession
    if (!sel.isNull() && !this._isEditing) {
      this._isEditing = true
      this.refs.sheet.showFormulaEditor(sheetSel.anchorRow, sheetSel.anchorCol)
      formulaEditorSession.resetHistory()
      console.log('_isEditing TRUE')
    }

  }

}
