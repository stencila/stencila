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
      tr.append($$('th').append(String(i)))
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
      tr.append($$('th').text(String(i)))
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
        let anchor = this.refs[`${anchorRow}_${anchorCol}`]
        if (anchor) {
          let anchorRect = getRelativeBoundingRect(anchor.el, this.el)
          styles.anchor.top = anchorRect.top
          styles.anchor.left = anchorRect.left
          styles.anchor.width = anchorRect.width
          styles.anchor.height = anchorRect.height
          styles.anchor.visibility = 'visible'
        }
        let [ulRow, ulCol] = [Math.max(startRow, state.startRow), Math.max(startCol, state.startCol)]
        let [lrRow, lrCol] = [Math.min(endRow, state.endRow), Math.min(endCol, state.endCol)]
        let ul = this.refs[`${ulRow}_${ulCol}`]
        let lr = this.refs[`${lrRow}_${lrCol}`]
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

        break
      }
      default:
        // nothing
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
    // TODO: get the anchor row/col
    let [rowIdx, colIdx] = this._getCellPositionForXY(e.clientX, e.clientY)
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
      let [rowIdx, colIdx] = this._getCellPositionForXY(e.clientX, e.clientY)
      if (rowIdx !== this._focusRow || colIdx !== this._focusCol) {
        this._focusRow = rowIdx
        this._focusCol = colIdx
        this._setSelection()
      }
    }
  }

  _onDblclick(e) {
    console.log('_onDblclick', e)
  }

  _getCustomResourceId() {
    return this.props.sheet.getName()
  }

  _setSelection() {
    // TODO: generate selection data from internal state
    let data = {
      type: 'range',
      anchorRow: this._anchorRow, anchorCol: this._anchorCol,
      focusRow: this._focusRow, focusCol: this._focusCol
    }
    this.context.editorSession.setSelection({
      type: 'custom',
      customType: 'sheet',
      data,
      surfaceId: this.getId()
    })
  }

  _getCellPositionForXY(clientX, clientY) {
    const state = this.state
    let offset = this.el.getOffset()
    let x = clientX - offset.left
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
    let colEls = this.refs.headRow.el.children
    let colIdx = state.startCol
    i = 1
    while (colIdx < state.endCol) {
      let rect = getRelativeBoundingRect(colEls[i], this.el)
      if (rect.left+rect.width > x) break
      colIdx++
      i++
    }
    // make sure that we provide indexes within the current viewport
    rowIdx = Math.max(state.startRow, Math.min(state.endRow, rowIdx))
    colIdx = Math.max(state.startCol, Math.min(state.endCol, colIdx))
    return [rowIdx, colIdx]
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
