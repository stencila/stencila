import {
  CustomSurface, getRelativeBoundingRect, platform, DefaultDOMElement,
  keys, clone
} from 'substance'
import SpreadsheetCellEditor from './SpreadsheetCellEditor'
import SheetView from './SheetView'
import SheetViewport from './SheetViewport'
import SheetScrollbar from './SheetScrollbar'
import SpreadsheetContextMenu from './SpreadsheetContextMenu'
import SpreadsheetClipboard from './SpreadsheetClipboard'
import { getRange } from './spreadsheetUtils'

export default class SpreadsheetComponent extends CustomSurface {

  getInitialState() {
    this._clipboard = new SpreadsheetClipboard(this.context.editorSession)
    this._viewport = new SheetViewport(this.props.sheet, this)
    // internal state used during cell editing
    this._isEditing = false
    this._cell = null
    // internal state used during selection
    this._isSelecting = false
    this._selection = {
      type: 'range',
      anchorRow: -1,
      anchorCol: -1,
      focusRow: -1,
      focusCol: -1
    }
    // TODO: we should think about using Component state instead
    return {}
  }

  didMount() {
    super.didMount()

    const editorSession = this.context.editorSession
    editorSession.on('render', this._onSelectionChange, this, {
      resource: 'selection'
    })
    editorSession.on('render', this._onDocumentChange, this, {
      resource: 'document'
    })
    // rerender the table view as soon the real element height is known
    this.refs.sheetView.rerender()
    // position selection overlays to reflect an initial selection
    this._positionSelection()
  }

  dispose() {
    super.dispose()

    this.context.editorSession.off(this)
  }

  didUpdate() {
    this._positionSelection()
  }

  render($$) {
    const sheet = this._getSheet()
    const viewport = this._viewport
    let el = $$('div').addClass('sc-spreadsheet')
    el.append(
      $$('textarea').addClass('se-keytrap').ref('keytrap')
        .css({ position: 'absolute', width: 0, height: 0 })
        .on('keydown', this._onKeyDown),
      $$('div').addClass('se-content').append(
        $$(SheetView, {
          sheet, viewport
        }).ref('sheetView')
      ),
      this._renderOverlay($$),
      this._renderCellEditor($$),
      this._renderRowContextMenu($$),
      this._renderColumnContextMenu($$),
      $$(SheetScrollbar, {
        sheet, viewport,
        axis: 'x'
      }).ref('scrollX'),
      $$(SheetScrollbar, {
        sheet, viewport,
        axis: 'y'
      }).ref('scrollY')
    )
    el.on('wheel', this._onWheel, this)
      .on('mousedown', this._onMousedown)
      .on('mousemove', this._onMousemove)
      .on('dblclick', this._onDblclick)
      .on('contextmenu', this._onContextMenu)
      .on('contextmenuitemclick', this._hideMenus)
      .on('copy', this._onCopy)
      .on('paste', this._onPaste)
      .on('cut', this._onCut)
    return el
  }

  resize() {
    this.refs.sheetView.rerender()
    this.refs.scrollX.rerender()
    this.refs.scrollY.rerender()
    this._positionSelection()
  }

  _renderCellEditor($$) {
    return $$(SpreadsheetCellEditor, { sheet: this._getSheet() })
      .ref('cellEditor')
      .css('display', 'none')
      .on('enter', this._onCellEditorEnter)
      .on('escape', this._onCellEditorEscape)
  }

  _renderOverlay($$) {
    let el = $$('div').addClass('se-overlay')
    el.append(
      $$('div').addClass('se-selection-anchor').ref('selAnchor').css('visibility', 'hidden'),
      $$('div').addClass('se-selection-range').ref('selRange').css('visibility', 'hidden'),
      $$('div').addClass('se-selection-columns').ref('selColumns').css('visibility', 'hidden'),
      $$('div').addClass('se-selection-rows').ref('selRows').css('visibility', 'hidden')
    )
    return el
  }

  _renderRowContextMenu($$) {
    const configurator = this.context.configurator
    let rowMenu = $$(SpreadsheetContextMenu, {
      toolPanel: configurator.getToolPanel('row-context-menu')
    }).ref('rowMenu')
      .addClass('se-context-menu')
      .css({ display: 'none' })
    return rowMenu
  }

  _renderColumnContextMenu($$) {
    const configurator = this.context.configurator
    let colMenu = $$(SpreadsheetContextMenu, {
      toolPanel: configurator.getToolPanel('column-context-menu')
    }).ref('columnMenu')
      .addClass('se-context-menu')
      .css({
        display: 'none'
      })
    return colMenu
  }

  _getCustomResourceId() {
    return this._getSheet().getName()
  }

    // called by SurfaceManager to render the selection plus setting the
  // DOM selection into a proper state
  rerenderDOMSelection() {
    // console.log('SpreadsheetComponent.rerenderDOMSelection()')
    this._positionSelection()
    // put the native focus into the keytrap so that we
    // receive keyboard events
    this.refs.keytrap.el.focus()
  }

  _getBoundingRect(rowIdx, colIdx) {
    return this.refs.sheetView.getBoundingRect(rowIdx, colIdx)
  }

  _getCellComponent(rowIdx, colIdx) {
    return this.refs.sheetView.getCellComponent(rowIdx, colIdx)
  }

  _positionSelection() {
    const sel = this.context.editorSession.getSelection()
    if (sel.surfaceId === this.getId()) {
      let styles = this._computeSelectionStyles(sel)
      this.refs.selAnchor.css(styles.anchor)
      this.refs.selRange.css(styles.range)
      this.refs.selColumns.css(styles.columns)
      this.refs.selRows.css(styles.rows)
    }
  }

  _computeSelectionStyles(sel) {
    const viewport = this._getViewport()
    const data = sel.data
    let styles = {
      anchor: { visibility: 'hidden' },
      range: { visibility: 'hidden' },
      columns: { visibility: 'hidden' },
      rows: { visibility: 'hidden' },
    }
    let anchorRow, anchorCol
    let ulRow, ulCol, lrRow, lrCol
    switch(data.type) {
      case 'range': {
        anchorRow = data.anchorRow
        anchorCol = data.anchorCol
        let focusRow = data.focusRow
        let focusCol = data.focusCol
        let startRow = anchorRow
        let startCol = anchorCol
        let endRow = focusRow
        let endCol = focusCol
        if (startRow > endRow) {
          [startRow, endRow] = [endRow, startRow]
        }
        if (startCol > endCol) {
          [startCol, endCol] = [endCol, startCol]
        }
        // don't render the selection if it is completely outside of the viewport
        if (endRow < viewport.startRow || startRow > viewport.endRow ||
            endCol < viewport.startCol || startCol > viewport.endCol ) {
          break
        }
        [ulRow, ulCol] = [Math.max(startRow, viewport.startRow), Math.max(startCol, viewport.startCol)]
        ;[lrRow, lrCol] = [Math.min(endRow, viewport.endRow), Math.min(endCol, viewport.endCol)]
        break
      }
      case 'columns': {
        anchorCol = data.anchorCol
        anchorRow = viewport.startRow
        let focusCol = data.focusCol
        let startCol = anchorCol
        let endCol = focusCol
        if (startCol > endCol) {
          [startCol, endCol] = [endCol, startCol]
        }
        [ulRow, ulCol] = [viewport.startRow, Math.max(startCol, viewport.startCol)]
        ;[lrRow, lrCol] = [viewport.endRow, Math.min(endCol, viewport.endCol)]
        break
      }
      case 'rows': {
        anchorRow = data.anchorRow
        anchorCol = viewport.startCol
        let focusRow = data.focusRow
        let startRow = anchorRow
        let endRow = focusRow
        if (startRow > endRow) {
          [startRow, endRow] = [endRow, startRow]
        }
        [ulRow, ulCol] = [Math.max(startRow, viewport.startRow), viewport.startCol]
        ;[lrRow, lrCol] = [Math.min(endRow, viewport.endRow), viewport.endCol]
        break
      }
      default:
        return styles
    }
    // TODO: We need to improve rendering for range selections
    // that are outside of the viewport
    let anchorRect = this._getBoundingRect(anchorRow, anchorCol)
    if (anchorRect.width && anchorRect.height) {
      Object.assign(styles, this._computeAnchorStyles(anchorRect))
    }
    let ulRect = this._getBoundingRect(ulRow, ulCol)
    let lrRect = this._getBoundingRect(lrRow, lrCol)
    Object.assign(styles, this._computeRangeStyles(ulRect, lrRect, data.type))
    return styles
  }

  _computeAnchorStyles(anchorRect) {
    let styles = { anchor: { visibility: 'hidden' } }
    if (anchorRect) {
      styles.anchor.top = anchorRect.top
      styles.anchor.left = anchorRect.left
      styles.anchor.width = anchorRect.width
      styles.anchor.height = anchorRect.height
      styles.anchor.visibility = 'visible'
    }
    return styles
  }

  _computeRangeStyles(ulRect, lrRect, mode) {
    let styles = {
      range: { visibility: 'hidden' },
      columns: { visibility: 'hidden' },
      rows: { visibility: 'hidden' }
    }

    // FIXME: in GDocs the background is only blue if the range is over multiple cells
    // TODO: the API does not state that the elements must be native elements here.
    //       IMO it should work with DOMElement in first place, and use native elements where necessary
    styles.range.top = ulRect.top
    styles.range.left = ulRect.left
    styles.range.width = lrRect.left + lrRect.width - styles.range.left
    styles.range.height = lrRect.top + lrRect.height - styles.range.top
    styles.range.visibility = 'visible'

    let cornerRect = getRelativeBoundingRect(this.refs.sheetView.getCornerComponent().el, this.el)

    if (mode === 'range' || mode === 'columns') {
      styles.columns.left = ulRect.left
      styles.columns.top = cornerRect.top
      styles.columns.height = cornerRect.height
      styles.columns.width = lrRect.left + lrRect.width - styles.columns.left
      styles.columns.visibility = 'visible'
    }

    if (mode === 'range' || mode === 'rows') {
      styles.rows.top = ulRect.top
      styles.rows.left = cornerRect.left
      styles.rows.width = cornerRect.width
      styles.rows.height = lrRect.top + lrRect.height - styles.rows.top
      styles.rows.visibility = 'visible'
    }

    return styles
  }

  _hideSelection() {
    this.refs.selection.css('visibility', 'hidden')
  }

  _getSelection() {
    return this.context.editorSession.getSelection().data
  }

  _nav(dr, dc, shift) {
    // throttle
    if (this._isNavigating) return
    this._isNavigating = true
    try {
      const editorSession = this.context.editorSession
      const viewport = this._getViewport()
      let data = this._getSelection()
      // TODO: move viewport if necessary
      let newFocusRow, newFocusCol
      if (!shift) {
        [newFocusRow, newFocusCol] = this._clamped(data.anchorRow+dr, data.anchorCol+dc)
        data.anchorRow = data.focusRow = newFocusRow
        data.anchorCol = data.focusCol = newFocusCol
      } else {
        [newFocusRow, newFocusCol] = this._clamped(data.focusRow+dr, data.focusCol+dc)
        data.focusRow = newFocusRow
        data.focusCol = newFocusCol
      }
      {
        let dr = 0
        let dc = 0
        if (newFocusRow < viewport.startRow) {
          dr = newFocusRow - viewport.startRow
        } else if (newFocusRow > viewport.endRow) {
          dr = newFocusRow - viewport.endRow
        }
        if(newFocusCol < viewport.startCol) {
          dc = newFocusCol - viewport.startCol
        } else if (newFocusCol > viewport.endCol) {
          dc = newFocusCol - viewport.endCol
        }
        if (dr || dc) {
          viewport.shift(dr, dc)
        }
      }
      editorSession.setSelection({
        type: 'custom',
        customType: 'sheet',
        data,
        surfaceId: this.getId()
      })
    } finally {
      this._isNavigating = false
    }
  }

  _clamped(rowIdx, colIdx) {
    const sheet = this._getSheet()
    const N = sheet.getRowCount()
    const M = sheet.getColumnCount()
    return [
      Math.max(0, Math.min(N-1, rowIdx)),
      Math.max(0, Math.min(M-1, colIdx)),
    ]
  }

  _setSelection() {
    let data = clone(this._selection)
    this.context.editorSession.setSelection({
      type: 'custom',
      customType: 'sheet',
      data,
      surfaceId: this.getId()
    })
  }

  _getSheet() {
    return this.props.sheet
  }

  _getViewport() {
    return this._viewport
  }

  _getTargetForEvent(e) {
    return this.refs.sheetView.getTargetForEvent(e)
  }

  /*
    This gets called when the user enters a cell.
    At this time it should be sure that the table cell
    is already rendered.
  */
  _openCellEditor(rowIdx, colIdx) {
    const cellEditor = this.refs.cellEditor
    let td = this._getCellComponent(rowIdx, colIdx)
    let rect = getRelativeBoundingRect(td.el, this.el)
    let cellComp = td.getChildAt(0)
    let cell = cellComp.props.node
    cellEditor.extendProps({ node: cell })
    cellEditor.css({
      display: 'block',
      top: rect.top,
      left: rect.left,
      "min-width": rect.width+'px',
      "min-height": rect.height+'px'
    })
    cellEditor.focus()
    this._isEditing = true
    this._cell = cell
  }

  _closeCellEditor() {
    const cellEditor = this.refs.cellEditor
    const cell = this._cell
    cellEditor.css({
      display: 'none',
      top: 0, left: 0
    })
    this.context.editorSession.transaction((tx) => {
      tx.set(cell.getTextPath(), cellEditor.getValue())
    })
    this._isEditing = false
    this._cell = null
  }

  _showRowMenu(e) {
    this._hideMenus()
    const rowMenu = this.refs.rowMenu
    let offset = this.el.getOffset()
    rowMenu.css({
      display: 'block',
      top: e.clientY - offset.top,
      left: e.clientX - offset.left
    })
  }

  _showColumnMenu(e) {
    this._hideMenus()
    const columnMenu = this.refs.columnMenu
    let offset = this.el.getOffset()
    columnMenu.css({
      display: 'block',
      top: e.clientY - offset.top,
      left: e.clientX - offset.left
    })
  }

  _hideMenus() {
    this.refs.rowMenu.css('display', 'none')
    this.refs.columnMenu.css('display', 'none')
  }

  _clearSelection() {
    const editorSession = this.context.editorSession
    let range = getRange(editorSession)
    editorSession.transaction((tx) => {
      tx.getDocument().clearRange(range.startRow, range.startCol, range.endRow, range.endCol)
    })
  }

  /* Event Handlers */

  _onSelectionChange(sel) {
    if (sel.surfaceId !== this.getId()) {
      this._hideSelection()
    }
  }

  _onDocumentChange(change) {
    if (change.hasUpdated('data')) {
      this.refs.sheetView.rerender()
    }
  }

  _onWheel(e) {
    e.stopPropagation()
    e.preventDefault()
    this._viewport.scroll(e.deltaX, e.deltaY)
    this._positionSelection()
  }

  _onMousedown(e) {
    // console.log('_onMousedown', e)
    e.stopPropagation()
    e.preventDefault()

    // close context menus
    this._hideMenus()

    // if editing a cell save the content
    if (this._isEditing) {
      this._closeCellEditor()
    }

    // TODO: do not update the selection if right-clicked and already having a selection

    if (platform.inBrowser) {
      DefaultDOMElement.wrap(window.document).on('mouseup', this._onMouseup, this, {
        once: true
      })
    }
    const sel = this._selection

    // console.log('_onMousedown', e)
    let target = this._getTargetForEvent(e)
    // console.log('... target', target)

    // TODO: move this into substance helper
    let isRightButton = false
    if ("which" in e) {
      isRightButton = (e.which === 3)
    } else if ("button" in e) {
      isRightButton = (e.button === 2)
    }
    if (isRightButton) {
      // TODO: improve right click handling
      // i.e. change the selection if target
      // is not within current selection
    } else {
      switch(target.type) {
        case 'cell': {
          this._isSelecting = true
          sel.type = 'range'
          sel.focusRow = target.rowIdx
          sel.focusCol = target.colIdx
          if (!e.shiftKey) {
            sel.anchorRow = sel.focusRow
            sel.anchorCol = sel.focusCol
          }
          this._setSelection()
          break
        }
        case 'column': {
          this._isSelecting = true
          sel.type = 'columns'
          sel.focusCol = target.colIdx
          if (!e.shiftKey) {
            sel.anchorCol = sel.focusCol
          }
          this._setSelection()
          break
        }
        case 'row': {
          this._isSelecting = true
          sel.type = 'rows'
          sel.focusRow = target.rowIdx
          if (!e.shiftKey) {
            sel.anchorRow = sel.focusRow
          }
          this._setSelection()
          break
        }
        default:
          //
      }
    }
  }

  _onMouseup(e) {
    e.stopPropagation()
    e.preventDefault()
    this._isSelecting = false
  }

  _onMousemove(e) {
    if (this._isSelecting) {
      const sheetView = this.refs.sheetView
      const sel = this._selection
      switch (sel.type) {
        case 'range': {
          let rowIdx = sheetView.getRowIndex(e.clientY)
          let colIdx = sheetView.getColumnIndex(e.clientX)
          if (rowIdx !== sel.focusRow || colIdx !== sel.focusCol) {
            sel.focusRow = rowIdx
            sel.focusCol = colIdx
            this._setSelection()
          }
          break
        }
        case 'columns': {
          let colIdx = sheetView.getColumnIndex(e.clientX)
          if (colIdx !== sel.focusCol) {
            sel.focusCol = colIdx
            this._setSelection()
          }
          break
        }
        case 'rows': {
          let rowIdx = sheetView.getRowIndex(e.clientY)
          if (rowIdx !== sel.focusRow) {
            sel.focusRow = rowIdx
            this._setSelection()
          }
          break
        }
        default:
          // should not happen
      }
    }
  }

  _onDblclick(e) {
    if (!this._isEditing) {
      const sheetView = this.refs.sheetView
      let rowIdx = sheetView.getRowIndex(e.clientY)
      let colIdx = sheetView.getColumnIndex(e.clientX)
      if (rowIdx > -1 && colIdx > -1) {
        this._openCellEditor(rowIdx, colIdx)
      }
    }
  }

  _onCellEditorEnter() {
    this._closeCellEditor()
    this._nav(1, 0)
  }

  _onCellEditorEscape() {
    const cellEditor = this.refs.cellEditor
    cellEditor.css({
      display: 'none',
      top: 0, left: 0
    })
    this._isEditing = false
    this._cell = null

    // HACK: resetting the selection
    const editorSession = this.context.editorSession
    editorSession.setSelection(editorSession.getSelection())
  }

  _onContextMenu(e) {
    // console.log('_onContextMenu()', e)
    e.preventDefault()
    e.stopPropagation()

    let target = this._getTargetForEvent(e)
    switch(target.type) {
      case 'cell': {
        console.info('TODO: implement cell context menu?')
        break
      }
      case 'row': {
        this._showRowMenu(e)
        break
      }
      case 'column': {
        this._showColumnMenu(e)
        break
      }
      default:
        //
    }
  }

  _onKeyDown(e) {
    let handled = false
    switch (e.keyCode) {
      case keys.LEFT:
        this._nav(0, -1, e.shiftKey)
        handled = true
        break
      case keys.RIGHT:
        this._nav(0, 1, e.shiftKey)
        handled = true
        break
      case keys.UP:
        this._nav(-1, 0, e.shiftKey)
        handled = true
        break
      case keys.DOWN:
        this._nav(1, 0, e.shiftKey)
        handled = true
        break
      case keys.ENTER: {
        let data = this._getSelection()
        this._openCellEditor(data.anchorRow, data.anchorCol)
        handled = true
        break
      }
      case keys.DELETE:
      case keys.BACKSPACE: {
        this._clearSelection()
        handled = true
        break
      }
      default:
        //
    }
    if (handled) {
      e.preventDefault()
      e.stopPropagation()
    }
  }

  _onCopy(e) {
    this._clipboard.onCopy(e)
  }

  _onPaste(e) {
    this._clipboard.onPaste(e)
  }

  _onCut(e) {
    this._clipboard.onCut(e)
  }

}
