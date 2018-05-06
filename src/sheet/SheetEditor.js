import {
  platform, DefaultDOMElement, AbstractEditor,
  Toolbar, EditorSession, DOMSelection,
  Configurator
} from 'substance'
import FormulaBar from './FormulaBar'
import FormulaEditor from './FormulaEditor'
import FunctionUsageCommand from '../shared/FunctionUsageCommand'
import FunctionUsageTool from '../shared/FunctionUsageTool'
import CodeEditorPackage from '../shared/CodeEditorPackage'
import { getCellLabel } from './sheetHelpers'
import { getRowCol, getCellState, isExpression} from '../shared/cellHelpers'
import CellRangesOverlay from './CellRangesOverlay'

export default class SheetEditor extends AbstractEditor {

  _initialize(props) {
    super._initialize(props)

    // a context for FormulaBar and FormulaEditor
    this._formulaEditorContext = this._createFormulaEditorContext()
    // true when the cursor is either in the FormularBar or the FormulaEditor
    this._isEditing = false

    const editorSession = props.editorSession
    if (platform.inBrowser) {
      DefaultDOMElement.wrap(window).on('resize', this._onResize, this)
    }
    editorSession.onUpdate('selection', this._onSelectionChange, this)
    this._formulaEditorContext.editorSession.onRender('selection', this._onCellEditorSelectionChange, this)
  }

  didMount() {
    super.didMount()
    this._postRender()

    this.handleActions({
      'updateCell': this._updateCell,
      'cancelCellEditing': this._cancelCellEditing,
      'editCell': this._editCell,
      'requestSelectionChange': this._requestSelectionChange,
      'selectAll': this._selectAll,
      'executeCommand': this._executeCommand
    })

    this._updateViewport()
  }


  _updateViewport() {
    let viewport = this.props.viewport
    if (viewport) {
      this.refs.sheet._viewport.update(viewport)
    }
  }

  getViewport() {
    return this.refs.sheet._viewport.toJSON()
  }

  /*
    An extra render cycle, once we know the sheet's dimensions
  */
  _postRender() {
    this._postrendering = true
    this.rerender()
    this._postrendering = false
    this._selectFirstCell()
  }

  /*
    Like in didMount we need to call _postRender when the component has been
    updated (e.g. new props). But we need to guard it, as the explicit in
    rerender also triggers a didUpdate call.
  */
  didUpdate() {
    if (!this._postrendering) {
      this._postRender()
      this._updateViewport()
    }
  }

  _selectFirstCell() {
    const editorSession = this.props.editorSession
    let sel = editorSession.getSelection().toJSON()

    if (!sel) {
      // Set the selection into the first cell
      // Doing this delayed to be in a new flow
      setTimeout(() => {
        editorSession.setSelection(
          this.getSheetComponent().selectFirstCell()
        )
      }, 0)
    } else {
      this.refs.sheet.rerenderDOMSelection()
    }

  }

  getChildContext() {
    const editorSession = this.props.editorSession
    const keyboardManager = this.keyboardManager
    const configurator = editorSession.getConfigurator()
    const issueManager = editorSession.getManager('issue-manager')
    const host = this.context.host
    return Object.assign({}, super.getChildContext(), {
      configurator,
      host,
      issueManager,
      keyboardManager
    })
  }

  getInitialState() {
    return {
      showContext: false,
      contextId: null,
      cellId: null
    }
  }

  _dispose() {
    super._dispose()
    const editorSession = this.props.editorSession
    if (platform.inBrowser) {
      DefaultDOMElement.wrap(window).off(this)
    }
    editorSession.off(this)
    this._formulaEditorContext.editorSession.off(this)
  }

  render($$) {
    let el = $$('div').addClass('sc-sheet-editor')
    el.on('keydown', super.onKeyDown)
    el.append(
      $$('div').addClass('se-main-section').append(
        this._renderToolpane($$),
        this._renderContent($$)
      )
    )

    if (this.props.contextComponent) {
      el.append(
        $$('div').addClass('se-context-pane').append(
          $$('div').addClass('se-context-pane-content').append(
            this.props.contextComponent
          )
        )
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
      }).ref('formulaBar'),
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
    const viewport = this.props.viewport
    const formulaEditorContext = this._formulaEditorContext
    // only rendering the sheet when mounted
    // so that we have real width and height
    if (this.isMounted()) {
      const SheetComponent = this.getComponent('sheet')
      return $$(SheetComponent, {
        viewport,
        sheet,
        overlays: [
          $$(FormulaEditor, {
            context: formulaEditorContext
          }).ref('formulaEditor')
            .css({
              position: 'absolute',
              display: 'none'
            })
        ],
        unclickableOverlays: [
          // a component that we use to highlight cell ranges
          // e.g. while editing a formula
          $$(CellRangesOverlay).ref('cellRanges')
        ],
      }).ref('sheet')
    } else {
      return $$('div')
    }
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
    const configurator = new Configurator()
    configurator.import(CodeEditorPackage)
    // TODO: let's see if we can generalize this, so that it can
    // go into the CodeEditorPackage
    configurator.addCommand('function-usage', FunctionUsageCommand, {
      commandGroup: 'prompt'
    })
    configurator.addTool('function-usage', FunctionUsageTool)
    configurator.addToolPanel('prompt', [
      {
        name: 'prompt',
        type: 'tool-group',
        showDisabled: false,
        commandGroups: ['prompt']
      }
    ])

    // a document with only one node used by cell editors
    // i.e. expression bar, or popover editor on enter
    let cellEditorDoc = configurator.createDocument()
    let node = cellEditorDoc.create({
      id: 'cell',
      type: 'cell',
      content: ''
    })
    let editorSession = new EditorSession(cellEditorDoc, { configurator })
    const self = this
    // provide an adapter for DOMSelection
    // TODO: either use a helper to create the DOMSelection or change DOMSelection's ctor to be better usable
    let domSelection = new DOMSelection({
      getDocument() { return cellEditorDoc },
      getSurfaceManager() { return editorSession.surfaceManager },
      getElement() { return self.getElement() }
    })
    return {
      configurator,
      editorSession,
      domSelection,
      node,
      markersManager: editorSession.markersManager,
      surfaceManager: editorSession.surfaceManager,
      commandManager: editorSession.commandManager,
      commandGroups: configurator.getCommandGroups(),
      tools: configurator.getTools(),
      functionManager: this.context.host.functionManager
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
      this._hideOverlays()
      formulaEditorSession.setSelection(null)
    }
  }

  _setReferenceSelection(reference) {
    const from = reference.split(':')[0]
    const to = reference.split(':')[1]
    const [startRow, startCol] = getRowCol(from)
    const [endRow, endCol] = to ? getRowCol(to) : [startRow, startCol]
    const sheetComp = this.getSheetComponent()
    let rect = sheetComp.getRectangleForRange({
      anchorRow: startRow,
      focusRow: endRow ? endRow : startRow,
      anchorCol: startCol,
      focusCol: endCol ? endCol: startCol
    })
    this.refs.cellRanges.setProps({ ranges: [rect] })
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
    const formulaSelection = formulaEditorSession.getSelection()
    if(formulaSelection && !formulaSelection.isNull()) {
      const cursorOffset = formulaSelection.start.offset
      const cell = formulaEditorSession.getDocument().get('cell')
      const cellState = getCellState(cell)
      const tokens = cellState.tokens
      const activeToken = tokens.find(token => {
        return token.type === 'cell' && cursorOffset >= token.start && cursorOffset <= token.end
      })
      if(activeToken) {
        const cellReference = activeToken.text
        this._setReferenceSelection(cellReference)
      } else {
        const sheetComp = this.getSheetComponent()
        sheetComp._hideSelection()
      }
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

  _hideOverlays() {
    this._hideFormulaEditor()
    this._hideCellRanges()
  }

  _hideCellRanges() {
    this.refs.cellRanges.css({ display: 'none' })
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
    const formulaEditorSession = this._formulaEditorContext.editorSession
    const formulaNode = this._formulaEditorContext.node
    if (initialValue) {
      this._setFormula(initialValue)
    }
    const path = formulaNode.getPath()
    const text = formulaNode.getText()
    const startOffset = text.length
    formulaEditorSession.setSelection({
      type: 'property',
      path,
      startOffset,
      surfaceId: 'formula-editor'
    })
  }

  _replaceEditorToken(from, to) {
    const formulaEditorSession = this._formulaEditorContext.editorSession
    const selection = formulaEditorSession.getSelection().toJSON()

    const cellState = formulaEditorSession.getDocument().get(['cell','state'])
    const tokens = cellState.tokens
    const activeToken = tokens.find(token => {
      return token.type === 'cell' && selection.startOffset >= token.start && selection.startOffset <= token.end
    })
    formulaEditorSession.transaction(tx => {
      if(activeToken) {
        selection.startOffset = activeToken.start
        selection.endOffset = activeToken.end
        tx.setSelection(selection)
      }
      const symbol = (from === to) ? from : from + ':' + to
      tx.insertText(symbol)
      if(!activeToken) {
        if(selection.startOffset === selection.endOffset) {
          selection.endOffset += symbol.length
        }
        tx.setSelection(selection)
      }
    })
  }

  _requestSelectionChange(newSelection) {
    const formulaEditorSession = this._formulaEditorContext.editorSession
    const cell = formulaEditorSession.getDocument().get('cell')
    const _isExpression = isExpression(cell.content)
    if (this._isEditing && _isExpression) {
      const selection = formulaEditorSession.getSelection().toJSON()
      const _insideExpression = selection.startOffset > 0
      if(_insideExpression) {
        const selData = newSelection.data
        const fromCell = getCellLabel(selData.anchorRow, selData.anchorCol)
        const toCell = getCellLabel(selData.focusRow, selData.focusCol)
        const sheetComp = this.getSheetComponent()
        this._replaceEditorToken(fromCell, toCell)
        let rect = sheetComp.getRectangleForRange(selData)
        this.refs.cellRanges.setProps({ ranges: [rect] })
      }
    } else {
      const editorSession = this.getEditorSession()
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

  _selectAll() {
    this._executeCommand('sheet:select-all')
  }

  _executeCommand(commandName, params) {
    // TODO: soon we will pull out CommandManager from EditorSession
    let commandManager = this.commandManager
    commandManager.executeCommand(commandName, params)
  }
}
