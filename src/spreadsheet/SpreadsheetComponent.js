import {
  CustomSurface, getRelativeBoundingRect, platform, DefaultDOMElement,
  Component, keys
} from 'substance'
import SpreadsheetLayout from './SpreadsheetLayout'
import SpreadsheetCell from './SpreadsheetCell'
import SpreadsheetCellEditor from './SpreadsheetCellEditor'

export default class SpreadsheetComponent extends CustomSurface {

  getInitialState() {
    // internal state which does trigger rerender
    this._layout = new SpreadsheetLayout(this.props.sheet)

    // TODO: this could be idiomatic states
    // internal state used during cell editing
    this._isEditing = false
    this._cell = null

    // internal state used during selection
    this._isSelecting = false
    this._anchorRow = -1
    this._anchorCol = -1
    this._focusRow = -1
    this._focusCol = -1

    const viewPort = this._layout.getViewport(0, 0)

    return Object.assign({}, viewPort)
  }

  didMount() {
    super.didMount()

    this.context.editorSession.on('render', this._onSelectionChange, this, {
      resource: 'selection'
    })
    // position initially, if the selection happens to be there from the beginning
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
    let el = $$('div').addClass('sc-spreadsheet')
    el.append(
      $$('textarea').addClass('se-box').ref('box')
        .css({ position: 'absolute', width: 0, height: 0 })
        .on('keydown', this._onKeyDown),
      $$('div').addClass('se-content').append(
        this._renderTable($$)
      ),
      this._renderOverlay($$),
      this._renderCellEditor($$),
      this._renderRowContextMenu($$),
      this._renderColumnContextMenu($$)
    )
    el.on('wheel', this._onWheel, this, { passive: true })
      .on('mousedown', this._onMousedown)
      .on('mousemove', this._onMousemove)
      .on('dblclick', this._onDblclick)
      .on('contextmenu', this._onContextMenu)
    return el
  }

  _renderTable($$) {
    let table = $$('table').css({
      width: this._layout.getWidth(this.state.startCol, this.state.endCol)
    })
    let nrows = this._layout.getRowCount()
    if (nrows > 0) {
      table.append(this._renderHeader($$))
      table.append(this._renderBody($$))
    }
    return table
  }

  _renderHeader($$) {
    let head = $$('thead')
    // TODO: in Sheets we want the classical column labels
    // in Datatable we want
    let tr = $$('tr').ref('headRow')
    tr.append($$('th').addClass('se-corner').ref('corner'))
    for (let i = this.state.startCol; i <= this.state.endCol; i++) {
      // TODO: map to ABC etc...
      tr.append(
        $$('th').append(String(i))
          .on('mousedown', this._onColumnMousedown)
          .on('contextmenu', this._onColumnContextMenu)
      )
    }
    head.append(tr)
    return head
  }

  _renderBody($$) {
    const state = this.state
    const sheet = this.props.sheet
    let body = $$('tbody').ref('body')
    for (let i = state.startRow; i <= state.endRow; i++) {
      let tr = $$('tr').ref(String(i))
      tr.append(
        $$('th').text(String(i))
          .on('mousedown', this._onRowMousedown)
          .on('contextmenu', this._onRowContextMenu)
      )
      for (let j = state.startCol; j <= state.endCol; j++) {
        const cell = sheet.getCell(i, j)
        let td = $$('td')
          .append($$(SpreadsheetCell, { node: cell }).ref(cell.id))
          .attr({
            'data-row': i,
            'data-col': j
          })
          .ref(`${i}_${j}`)
        tr.append(td)
      }
      body.append(tr)
    }
    return body
  }

  _renderCellEditor($$) {
    return $$(SpreadsheetCellEditor, { sheet: this.props.sheet })
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
    let rowMenu = $$(RowMenu).ref('rowMenu').addClass('se-context-menu')
    rowMenu.css({
      display: 'none'
    })
    return rowMenu
  }

  _renderColumnContextMenu($$) {
    let colMenu = $$(ColumnMenu).ref('columnMenu').addClass('se-context-menu')
    colMenu.css({
      display: 'none'
    })
    return colMenu
  }

    // called by SurfaceManager to render the selection plus setting the
  // DOM selection into a proper state
  rerenderDOMSelection() {
    // console.log('SpreadsheetComponent.rerenderDOMSelection()')
    this._positionSelection()
    this.refs.box.el.focus()
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

  _getCell(rowIdx, colIdx) {
    return this.refs[`${rowIdx}_${colIdx}`]
  }

  _computeSelectionStyles(sel) {
    const state = this.state
    const data = sel.data
    let styles = {
      anchor: { visibility: 'hidden' },
      range: { visibility: 'hidden' },
      columns: { visibility: 'hidden' },
      rows: { visibility: 'hidden' },
    }
    switch(data.type) {
      case 'range': {
        let { anchorRow, anchorCol, focusRow, focusCol } = data
        let startRow = anchorRow
        let startCol = anchorCol
        let endRow = focusRow
        let endCol = focusCol
        if (startRow > endRow) [startRow, endRow] = [endRow, startRow]
        if (startCol > endCol) [startCol, endCol] = [endCol, startCol]
        // don't render the selection if it is completely outside of the viewport
        if (endRow < state.startRow || startRow > state.endRow ||
            endCol < state.startCol || startCol > state.endCol ) {
          break
        }
        let [ulRow, ulCol] = [Math.max(startRow, state.startRow), Math.max(startCol, state.startCol)]
        let [lrRow, lrCol] = [Math.min(endRow, state.endRow), Math.min(endCol, state.endCol)]

        let anchor = this._getCell(anchorRow, anchorCol)
        let ul = this._getCell(ulRow, ulCol)
        let lr = this._getCell(lrRow, lrCol)

        Object.assign(styles, this._computeAnchorStyles(anchor))
        Object.assign(styles, this._computeRangeStyles(ul, lr, data.type))
        break
      }
      case 'columns': {
        let { anchorCol, focusCol } = data
        let startCol = anchorCol
        let endCol = focusCol
        if (startCol > endCol) [startCol, endCol] = [endCol, startCol]

        let [ulRow, ulCol] = [state.startRow, Math.max(startCol, state.startCol)]
        let [lrRow, lrCol] = [state.endRow, Math.min(endCol, state.endCol)]

        let anchor = this._getCell(state.startRow, anchorCol)
        let ul = this._getCell(ulRow, ulCol)
        let lr = this._getCell(lrRow, lrCol)

        Object.assign(styles, this._computeAnchorStyles(anchor))
        Object.assign(styles, this._computeRangeStyles(ul, lr, data.type))
        break
      }
      case 'rows': {
        let { anchorRow, focusRow } = data
        let startRow = anchorRow
        let endRow = focusRow
        if (startRow > endRow) [startRow, endRow] = [endRow, startRow]

        let [ulRow, ulCol] = [Math.max(startRow, state.startRow), state.startCol]
        let [lrRow, lrCol] = [Math.min(endRow, state.endRow), state.endCol]

        let anchor = this._getCell(anchorRow, state.startCol)
        let ul = this._getCell(ulRow, ulCol)
        let lr = this._getCell(lrRow, lrCol)

        Object.assign(styles, this._computeAnchorStyles(anchor))
        Object.assign(styles, this._computeRangeStyles(ul, lr, data.type))
        break
      }
      default:
        // nothing
    }

    return styles
  }

  _computeAnchorStyles(anchor) {
    let styles = { anchor: { visibility: 'hidden' } }
    if (anchor) {
      let anchorRect = getRelativeBoundingRect(anchor.el, this.el)
      styles.anchor.top = anchorRect.top
      styles.anchor.left = anchorRect.left
      styles.anchor.width = anchorRect.width
      styles.anchor.height = anchorRect.height
      styles.anchor.visibility = 'visible'
    }
    return styles
  }

  _computeRangeStyles(ul, lr, mode) {
    let styles = {
      range: { visibility: 'hidden' },
      columns: { visibility: 'hidden' },
      rows: { visibility: 'hidden' }
    }

    if (!ul || !lr) {
      console.error('FIXME: there is an error in retrieving the selected cell elements')
    } else {
      // FIXME: in GDocs the background is only blue if the range is over multiple cells
      // TODO: the API does not state that the elements must be native elements here.
      //       IMO it should work with DOMElement in first place, and use native elements where necessary
      let ulRect = getRelativeBoundingRect(ul.el, this.el)
      let lrRect = getRelativeBoundingRect(lr.el, this.el)
      styles.range.top = ulRect.top
      styles.range.left = ulRect.left
      styles.range.width = lrRect.left + lrRect.width - styles.range.left
      styles.range.height = lrRect.top + lrRect.height - styles.range.top
      styles.range.visibility = 'visible'

      let cornerRect = getRelativeBoundingRect(this.refs.corner.el, this.el)

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
    }

    return styles
  }

  _hideSelection() {
    this.refs.selection.css('visibility', 'hidden')
  }

  _onSelectionChange(sel) {
    if (sel.surfaceId !== this.getId()) {
      this._hideSelection()
    }
  }

  _onWheel(e) {
    e.stopPropagation()
    e.preventDefault()
    let deltaX = _step(e.deltaX)
    let deltaY = _step(e.deltaY)
    if (deltaX || deltaY) {
      let newStartCol = this.state.startCol + deltaX
      let newStartRow = this.state.startRow + deltaY
      this.extendState(this._layout.getViewport(newStartRow, newStartCol))
    }
  }

  _onMousedown(e) {
    // console.log('_onMousedown', e)
    e.stopPropagation()
    e.preventDefault()

    this._hideMenus()

    if (this._isEditing) {
      this._closeCellEditor()
    }

    // TODO: do not update the selection if right-clicked and already having a selection

    if (platform.inBrowser) {
      DefaultDOMElement.wrap(window.document).on('mouseup', this._onMouseup, this, {
        once: true
      })
    }
    this._isSelecting = true
    let [rowIdx, colIdx] = this._getCellPositionForXY(e.clientX, e.clientY)
    this._selType = 'range'
    this._anchorRow = this._focusRow = rowIdx
    this._anchorCol = this._focusCol = colIdx
    this._setSelection()
  }

  _onMouseup(e) {
    e.stopPropagation()
    e.preventDefault()
    this._isSelecting = false
  }

  _onMousemove(e) {
    if (this._isSelecting) {
      // console.log('_onMousemove', e)
      switch (this._selType) {
        case 'range': {
          let [rowIdx, colIdx] = this._getCellPositionForXY(e.clientX, e.clientY)
          if (rowIdx !== this._focusRow || colIdx !== this._focusCol) {
            this._focusRow = rowIdx
            this._focusCol = colIdx
            this._setSelection()
          }
          break
        }
        case 'columns': {
          let colIdx = this._getColumnIndex(e.clientX)
          if (colIdx !== this._focusCol) {
            this._focusCol = colIdx
            this._setSelection()
          }
          break
        }
        case 'rows': {
          let rowIdx = this._getRowIndex(e.clientY)
          if (rowIdx !== this._focusRow) {
            this._focusRow = rowIdx
            this._setSelection()
          }
          break
        }
        default:
          // should not happen
      }
    }
  }

  _onColumnMousedown(e) {
    // console.log('_onColumnMousedown', e)
    e.preventDefault()
    e.stopPropagation()
    this._hideMenus()
    if (this._isEditing) {
      this._closeCellEditor()
    }
    if (platform.inBrowser) {
      DefaultDOMElement.wrap(window.document).on('mouseup', this._onMouseup, this, {
        once: true
      })
    }
    this._isSelecting = true
    this._selType = 'columns'
    this._anchorCol = this._focusCol = this._getColumnIndex(e.clientX)
    this._setSelection()
  }

  _onRowMousedown(e) {
    // console.log('_onRowMousedown', e)
    e.preventDefault()
    e.stopPropagation()
    this._hideMenus()
    if (this._isEditing) {
      this._closeCellEditor()
    }
    if (platform.inBrowser) {
      DefaultDOMElement.wrap(window.document).on('mouseup', this._onMouseup, this, {
        once: true
      })
    }
    this._isSelecting = true
    this._selType = 'rows'
    this._anchorRow = this._focusRow = this._getRowIndex(e.clientY)
    this._setSelection()
  }

  _onDblclick(e) {
    const [rowIdx, colIdx] = this._getCellPositionForXY(e.clientX, e.clientY, 'strict')
    if (rowIdx > -1 && colIdx > -1) {
      this._openCellEditor(rowIdx, colIdx)
    }
  }

  _onCellEditorEnter() {
    this._closeCellEditor()
  }

  _onCellEditorEscape() {
    const cellEditor = this.refs.cellEditor
    cellEditor.css({
      display: 'none',
      top: 0, left: 0
    })
    this._isEditing = false
    this._cell = null
  }

  _onContextMenu(e) {
    // console.log('_onCellContextMenu()', e)
    e.preventDefault()
    e.stopPropagation()
  }

  _onRowContextMenu(e) {
    console.log('_onRowContextMenu()', e)
    e.preventDefault()
    e.stopPropagation()
    this._showRowMenu(e)
  }

  _onColumnContextMenu(e) {
    // console.log('_onColumnContextMenu()', e)
    e.preventDefault()
    e.stopPropagation()
    this._showColumnMenu(e)
  }

  _onKeyDown(e) {
    switch (e.keyCode) {
      case keys.LEFT:
        this._nav(0, -1, e.shiftKey)
        break
      case keys.RIGHT:
        this._nav(0, 1, e.shiftKey)
        break
      case keys.UP:
        this._nav(-1, 0, e.shiftKey)
        break
      case keys.DOWN:
        this._nav(1, 0, e.shiftKey)
        break
      case keys.ENTER: {
        let data = this._getSelection()
        if (data.anchorRow === data.focusRow && data.anchorCol === data.focusCol) {
          this._openCellEditor(data.anchorRow, data.anchorCol)
        } else {
          this._nav(1, 0)
        }
        break
      }
        break
      default:
        //
    }
  }

  _getSelection() {
    return this.context.editorSession.getSelection().data
  }

  _nav(dr, dc, shift) {
    let data = this._getSelection()
    // TODO: move viewport if necessary
    if (!shift) {
      data.anchorRow += dr
      data.anchorCol += dc
      data.focusRow = data.anchorRow
      data.focusCol = data.anchorCol
    } else {
      data.focusRow += dr
      data.focusCol += dc
    }
    this.context.editorSession.setSelection({
      type: 'custom',
      customType: 'sheet',
      data,
      surfaceId: this.getId()
    })
  }

  _getCustomResourceId() {
    return this.props.sheet.getName()
  }

  _setSelection() {
    let data
    switch (this._selType) {
      case 'range': {
        data = {
          type: 'range',
          anchorRow: this._anchorRow, anchorCol: this._anchorCol,
          focusRow: this._focusRow, focusCol: this._focusCol
        }
        break
      }
      case 'columns': {
        data = {
          type: 'columns',
          anchorCol: this._anchorCol,
          focusCol: this._focusCol
        }
        break
      }
      case 'rows': {
        data = {
          type: 'rows',
          anchorRow: this._anchorRow,
          focusRow: this._focusRow
        }
        break
      }
      default:
        throw new Error('Invalid type.')
    }
    this.context.editorSession.setSelection({
      type: 'custom',
      customType: 'sheet',
      data,
      surfaceId: this.getId()
    })
  }

  _getCellPositionForXY(clientX, clientY, strict) {
    let rowIdx = this._getRowIndex(clientY, strict)
    let colIdx = this._getColumnIndex(clientX, strict)
    return [rowIdx, colIdx]
  }

  _getRowIndex(clientY, strict) {
    const state = this.state
    let offset = this.el.getOffset()
    let y = clientY - offset.top
    // for now we always search without any trickery
    // could be improved using caching or a tree datastructure to find positions more quickly
    let bodyEl = this.refs.body.el
    let rowEls = bodyEl.children
    if (strict) {
      let rect = getRelativeBoundingRect(bodyEl, this.el)
      if (rect.top > y || rect.top + rect.height < y) return -1
    }
    let i = 0
    let rowIdx = state.startRow
    while (rowIdx < state.endRow) {
      let rect = getRelativeBoundingRect(rowEls[i], this.el)
      if (rect.top+rect.height > y) break
      rowIdx++
      i++
    }
    // make sure that we provide indexes within the current viewport
    rowIdx = Math.max(state.startRow, Math.min(state.endRow, rowIdx))
    return rowIdx
  }

  _getColumnIndex(clientX, strict) {
    const state = this.state
    let offset = this.el.getOffset()
    let x = clientX - offset.left
    let colEls = this.refs.headRow.el.children
    let i = 1
    let colIdx = state.startCol
    while (colIdx < state.endCol) {
      let rect = getRelativeBoundingRect(colEls[i], this.el)
      if (strict) {
        if (i === 1 && rect.left > x) return -1
        if (i === state.endCol-1 && rect.left+rect.width < x) return -1
      }
      if (rect.left+rect.width > x) break
      colIdx++
      i++
    }
    // make sure that we provide indexes within the current viewport
    colIdx = Math.max(state.startCol, Math.min(state.endCol, colIdx))
    return colIdx
  }

  _openCellEditor(rowIdx, colIdx) {
    const cellEditor = this.refs.cellEditor
    let td = this._getCell(rowIdx, colIdx)
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

}

class RowMenu extends Component {

  render($$) {
    let el = $$('div').addClass('sc-spreadsheet-row-menu')
    el.append($$('div').append('insert above').on('click', this._onInsertAbove))
    el.append($$('div').append('insert below').on('click', this._onInsertBelow))
    el.on('mousedown', _prevent)
    return el
  }

  _onInsertAbove(e) {
    e.preventDefault()
    e.stopPropagation()
    console.log('Insert above')
  }

  _onInsertBelow(e) {
    e.preventDefault()
    e.stopPropagation()
    console.log('Insert below')
  }

}

class ColumnMenu extends Component {

  render($$) {
    let el = $$('div').addClass('sc-spreadsheet-columnd-menu')
    el.append($$('div').append('insert left').on('click', this._onInsertLeft))
    el.append($$('div').append('insert right').on('click', this._onInsertRight))
    el.on('mousedown', _prevent)
    return el
  }

  _onInsertLeft(e) {
    e.preventDefault()
    e.stopPropagation()
    console.log('Insert left')
  }

  _onInsertRight(e) {
    e.preventDefault()
    e.stopPropagation()
    console.log('Insert right')
  }

}

function _prevent(e) {
  e.preventDefault()
  e.stopPropagation()
}

// signum with epsilon
const EPSILON = 1
function _step(x) {
  let abs = Math.abs(x)
  if (abs > EPSILON) {
    let sgn = Math.sign(x)
    // return sgn * Math.ceil(abs / 20)
    return sgn
  }
  return 0
}
