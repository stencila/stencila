import {
  platform, DefaultDOMElement, AbstractEditor,
  Toolbar, EditorSession, DOMSelection
} from 'substance'
import SheetContextSection from './SheetContextSection'
import SheetStatusBar from './SheetStatusBar'
import FormulaBar from './FormulaBar'
import FormulaEditor from './FormulaEditor'

export default class SheetEditor extends AbstractEditor {

  constructor(...args) {
    super(...args)
    // a context for FormulaBar and FormulaEditor
    this._formulaEditorContext = this._createFormulaEditorContext()
    // true when the cursor is either in the FormularBar or the FormulaEditor
    this._isEditing = false

    this.handleActions({
      'updateCell': this._updateCell,
      'cancelCellEditing': this._cancelCellEditing,
      'editCell': this._editCell,
      'requestSelectionChange': this._requestSelectionChange,
      'selectAll': this._selectAll,
      'executeCommand': this._executeCommand
    })
  }

  getChildContext() {
    const editorSession = this.props.editorSession
    const keyboardManager = this.keyboardManager
    const issueManager = editorSession.getManager('issue-manager')
    const host = this.props.host
    const configurator = editorSession.getConfigurator()
    return Object.assign({}, super.getChildContext(), {
      configurator,
      host,
      issueManager,
      keyboardManager
    })
  }

  getInitialState() {
    let sheetState = this._getSheetState()
    return {
      showContext: false,
      contextId: null,
      cellId: null,
      displayMode: sheetState.displayMode
    }
  }

  didMount() {
    super.didMount()

    const editorSession = this.props.editorSession
    if (platform.inBrowser) {
      DefaultDOMElement.wrap(window).on('resize', this._onResize, this)
    }
    editorSession.onUpdate('selection', this._onSelectionChange, this)
    editorSession.onUpdate('document', this._onSheetStateChange, this, {
      path: ['sheet.state']
    })

    this._formulaEditorContext.editorSession.onUpdate('selection', this._onCellEditorSelectionChange, this)

    this.rerender()

    // set the selection into the first cell
    // Doing this delayed to be in a new flow
    setTimeout(() => {
      editorSession.setSelection(
        this.getSheetComponent().selectFirstCell()
      )
    }, 0)
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
    el.addClass('sm-display-mode-'+this.state.displayMode)
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
      $$(FormulaBar, {
        node: this._formulaEditorContext.node,
        context: this._formulaEditorContext
      }),
      $$(Toolbar, {
        toolPanel: configurator.getToolPanel('toolbar')
      }).ref('toolbar')
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
        overlays: [
          $$(FormulaEditor, {
            context: formulaEditorContext
          })
            .ref('formulaEditor')
            .css({
              position: 'absolute',
              display: 'none'
            })
        ]
      }).ref('sheet')
        // LEGACY
        // TODO: the displayMode is app specific
        // so it should be set on the sc-sheet-editor
        // and the CSS should be reflect this
        .addClass('sm-mode-'+this.state.displayMode)
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

  getSheetComponent() {
    return this.refs.sheet
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
          this.refs.sheet.forceUpdate()
        })
      }
    }
  }

  _getSheetSelection() {
    return this.getEditorSession().getSelection().data || {}
  }

  _onSelectionChange(sel) {
    // TODO: what to do if the sheet seleciton is null?
    if (!sel) return

    let formulaEditorSession = this._formulaEditorContext.editorSession
    let cell = this._getAnchorCell()
    if (cell) {
      this._setFormula(cell.textContent)
    }
    if (this._isEditing) {
      this._isEditing = false
      this._hideFormulaEditor()
      formulaEditorSession.setSelection(null)
    }
  }

  _onCellEditorSelectionChange(sel) {
    let sheetSel = this._getSheetSelection()
    let formulaEditorSession = this._formulaEditorContext.editorSession
    if (!sel.isNull() && !this._isEditing) {
      this._isEditing = true
      this._currentSelection = this.getEditorSession().getSelection()
      this._showFormulaEditor(sheetSel.anchorRow, sheetSel.anchorCol)
      formulaEditorSession.resetHistory()
    }
  }

  _onSheetStateChange() {
    const sheet = this.props.editorSession.getDocument()
    const sheetState = sheet.getState()
    if (this.state.displayMode !== sheetState.displayMode) {
      this.getSheetComponent().forceUpdate()
      this.extendState({
        displayMode: sheetState.displayMode
      })
    }
  }

  /*
    This gets called when the user starts editing a cell
    At this time it should be sure that the table cell
    is already rendered.
  */
  _showFormulaEditor(rowIdx, colIdx) {
    const formulaEditor = this.refs.formulaEditor
    const sheetComponent = this.getSheetComponent()
    // only show if we actually get a rectangle
    // e.g. this is null if the cell is not in the
    // viewport
    let rect = sheetComponent.getCellRect(rowIdx, colIdx)
    if (rect) {
      formulaEditor.css({
        display: 'block',
        position: 'absolute',
        top: rect.top,
        left: rect.left,
        "min-width": rect.width+'px',
        "min-height": rect.height+'px'
      })
    } else {
      formulaEditor.css({
        display: 'none'
      })
    }
  }

  _hideFormulaEditor() {
    const formulaEditor = this.refs.formulaEditor
    formulaEditor.css({
      display: 'none',
      top: 0, left: 0
    })
  }

  _setFormula(val, sel) {
    let context = this._formulaEditorContext
    let node = context.node
    let formulaEditorSession = this._formulaEditorContext.editorSession
    formulaEditorSession.transaction(tx => {
      tx.set(node.getPath(), val)
      tx.setSelection(sel)
    })
  }

  _cancelCellEditing() {
    // just renew the the selection
    let editorSession = this.getEditorSession()
    editorSession.setSelection(editorSession.getSelection())
  }

  /*
    Request inline editor
  */
  _editCell(initialValue) {
    let formulaEditorSession = this._formulaEditorContext.editorSession
    let formulaNode = this._formulaEditorContext.node
    if (initialValue) {
      this._setFormula(initialValue)
    }
    let pos = formulaNode.textContent.length
    formulaEditorSession.setSelection({
      type: 'property',
      path: formulaNode.getPath(),
      startOffset: pos,
      surfaceId: 'formula-editor'
    })
  }

  _requestSelectionChange(newSelection) {
    let editorSession = this.getEditorSession()
    if (this._isEditing) {
      console.info('TODO: Implement range selector', newSelection)
    } else {
      editorSession.setSelection(newSelection)
    }
  }

  _updateCell() {
    let editorSession = this.getEditorSession()
    let cell = this._getAnchorCell()
    let oldValue = cell.getText()
    let newValue = this._formulaEditorContext.node.getText()

    let newSel = this.refs.sheet.shiftSelection(1, 0, false)
    // skip if there is no change
    if (oldValue !== newValue) {
      // collapsing the selection to the anchor cell
      // so that on undo/redo only the change cell is selected
      let selBefore = this._currentSelection
      selBefore.data.focusRow = selBefore.data.anchorRow
      selBefore.data.focusCol = selBefore.data.anchorCol
      // HACK: need to set the selection 'silently' so that
      // this works fine with undo/redo (-> before state)
      editorSession._setSelection(selBefore)
      editorSession.transaction(tx => {
        tx.set(cell.getPath(), newValue)
      })
    }
    // setting the selection in the transaction
    // leads to an inintuitiv undo/redo behavior
    // thus we are updating the selection in an extra update here
    editorSession.setSelection(newSel)
  }

  _getAnchorCell() {
    let sel = this._getSheetSelection()
    return this.getDocument().getCell(sel.anchorRow, sel.anchorCol)
  }

  _getSheetState() {
    const editorSession = this.props.editorSession
    const sheet = editorSession.getDocument()
    return sheet.getState()
  }

  _selectAll() {
    this._executeCommand('select-all')
  }

  _executeCommand(commandName, params) {
    // TODO: soon we will pull out CommandManager from EditorSession
    let commandManager = this.commandManager
    commandManager.executeCommand(commandName, params)
  }
}
