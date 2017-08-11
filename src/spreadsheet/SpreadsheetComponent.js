import { CustomSurface, getRelativeBoundingRect, platform, DefaultDOMElement } from 'substance'
import SpreadsheetLayout from './SpreadsheetLayout'
import SpreadsheetCell from './SpreadsheetCell'

export default class SpreadsheetComponent extends CustomSurface {

  getInitialState() {
    // internal state which does trigger rerender
    this._layout = new SpreadsheetLayout(this.props.sheet)
    this._isSelecting = false

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
      $$('div').addClass('se-content').append(
        this._renderTable($$)
      ),
      this._renderOverlay($$)
    )
    el.on('wheel', this._onWheel)
      .on('mousedown', this._onMousedown)
      .on('mousemove', this._onMousemove)
      .on('dblclick', this._onDblclick)
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
        $$('th').append(String(i)).on('mousedown', this._onColumnMousedown)
      )
    }
    head.append(tr)
    return head
  }

  _renderBody($$) {
    const sheet = this.props.sheet
    const data = sheet.find('data')
    const rows = data.children
    let body = $$('tbody').ref('body')
    for (let i = this.state.startRow; i <= this.state.endRow; i++) {
      const row = rows[i]
      let tr = $$('tr').ref(String(i))
      tr.append(
        $$('th').text(String(i))
          .on('mousedown', this._onRowMousedown)
      )
      let cells = row.children
      for (let j = this.state.startCol; j <= this.state.endCol; j++) {
        const cell = cells[j]
        tr.append(
          $$('td').append(
            $$(SpreadsheetCell, { node: cell }).ref(cell.id)
          ).attr({
            'data-row': i,
            'data-col': j
          }).ref(`${i}_${j}`)
        )
      }
      body.append(tr)
    }
    return body
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

    // called by SurfaceManager to render the selection plus setting the
  // DOM selection into a proper state
  rerenderDOMSelection() {
    // console.log('SpreadsheetComponent.rerenderDOMSelection()')
    this._positionSelection()
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
        Object.assign(styles, this._computeRangeStyles(ul, lr))
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
        Object.assign(styles, this._computeRangeStyles(ul, lr))
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
        Object.assign(styles, this._computeRangeStyles(ul, lr))
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

  _computeRangeStyles(ul, lr) {
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
      styles.columns.left = ulRect.left
      styles.columns.top = cornerRect.top
      styles.columns.height = cornerRect.height
      styles.columns.width = lrRect.left + lrRect.width - styles.columns.left
      styles.columns.visibility = 'visible'

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
    console.log('_onColumnMousedown', e)
    e.preventDefault()
    e.stopPropagation()
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
    console.log('_onRowMousedown', e)
    e.preventDefault()
    e.stopPropagation()
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
    console.log('_onDblclick', e)
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

  _getCellPositionForXY(clientX, clientY) {
    let rowIdx = this._getRowIndex(clientY)
    let colIdx = this._getColumnIndex(clientX)
    return [rowIdx, colIdx]
  }

  _getRowIndex(clientY) {
    const state = this.state
    let offset = this.el.getOffset()
    let y = clientY - offset.top
    // for now we always search without any trickery
    // could be improved using caching or a tree datastructure to find positions more quickly
    let rowIdx = state.startRow
    let rowEls = this.refs.body.el.children
    let i = 0
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

  _getColumnIndex(clientX) {
    const state = this.state
    let offset = this.el.getOffset()
    let x = clientX - offset.left
    let colEls = this.refs.headRow.el.children
    let colIdx = state.startCol
    let i = 1
    while (colIdx < state.endCol) {
      let rect = getRelativeBoundingRect(colEls[i], this.el)
      if (rect.left+rect.width > x) break
      colIdx++
      i++
    }
    // make sure that we provide indexes within the current viewport
    colIdx = Math.max(state.startCol, Math.min(state.endCol, colIdx))
    return colIdx
  }

}

// signum with epsilon
const EPSILON = 1
function _step(x) {
  let abs = Math.abs(x)
  if (abs > EPSILON) {
    let sgn = Math.sign(x)
    return sgn * Math.ceil(abs / 20)
  }
  return 0
}
