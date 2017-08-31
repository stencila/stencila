import {
  Component, isNumber, isFunction
} from 'substance'
import SpreadsheetCell from './SpreadsheetCell'

const D = 30

export default class TableView extends Component {

  getInitialState() {
    this._viewport = {
      // fictive scroll position: instead of real scroll
      // coordinates we apply a simple heuristic,
      // using a fixed height and width for every column
      // and a fictive position within this model
      dx: 0,
      dy: 0,
      // this is always the cell in the top-left corner
      startRow: 0,
      startCol: 0,
      // this is always the cell in the bottom-right corner
      // which is fully visible
      endRow: 0,
      endCol: 0
    }
    return {}
  }

  // shouldRerender() {
  //   // TODO: only rerender when the surrounding dimensions have changed
  //   return false
  // }

  didMount() {
    this._limitViewport()
  }

  didUpdate() {
    this._limitViewport()
  }

  render($$) {
    let el = $$('table')

    const sheet = this._getSheet()
    const N = sheet.getRowCount()
    const M = sheet.getColumnCount()
    const W = this._getWidth()
    const H = this._getHeight()
    const viewport = this._viewport
    const startRow = viewport.startRow
    const startCol = viewport.startCol

    let head = $$('tr').ref('head')
    // fill columns
    head.append($$('th').addClass('se-corner').ref('corner'))
    // HACK: 50px is currently the width of the label column
    // should be computed dynamically
    let width = 50
    let endCol = startCol
    for(let colIdx = startCol; colIdx < M; colIdx++, endCol++) {
      let w = sheet.getColumnWidth(colIdx)
      head.append(
        $$('th').text(String(colIdx))
          .css({ width: w })
      )
      width += w
      if (width > W) break
    }
    el.append(head)
      .css({ width })
    // stay within table bounds
    endCol = Math.min(M-1, endCol)

    let body = $$('tbody').ref('body')
    // HACK: 20px is currently the height of the header row
    // should be computed dynamically
    let height = 20
    let endRow = startRow
    for(let rowIdx = startRow; rowIdx < N; rowIdx++, endRow++) {
      let h = sheet.getRowHeight(rowIdx)
      body.append(
        this._renderRow($$, rowIdx, startCol, endCol)
      )
      height += h
      if (height > H) break
    }
    el.append(body)
    // stay within table bounds
    endRow = Math.min(N-1, endRow)
    // Note: the viewport will be limited later
    // considering the real row heights
    this._viewport.endRow = endRow
    this._viewport.endCol = endCol
    return el
  }

  _renderRow($$, rowIdx, startCol, endCol) {
    const sheet = this._getSheet()
    let tr = $$('tr').ref(String(rowIdx))
    tr.append(
      $$('th').text(String(rowIdx))
    )
    for (let j = startCol; j <= endCol; j++) {
      const cell = sheet.getCell(rowIdx, j)
      let td = $$('td')
        .append(
          $$(SpreadsheetCell, { node: cell }).ref(cell.id)
        ).attr({
          'data-row': rowIdx,
          'data-col': j
        })
      tr.append(td)
    }
    return tr
  }

  _limitViewport() {
    const H = this._getHeight()
    const bodyEl = this.refs.body.el
    const headEl = this.refs.head.el

    let viewport = this._viewport
    let endRow = viewport.startRow
    let it = bodyEl.getChildNodeIterator()
    let height = headEl.getHeight()
    while (it.hasNext()) {
      let rowEl = it.next()
      let h = rowEl.getHeight()
      if (height + h > H) break
      endRow++
      height += h
    }
    console.log('Limiting viewport %s -> %s', viewport.endRow, endRow)
    viewport.endRow = endRow

  }

  getTargetForEvent(e) {
    let rect = this._getRect()
    let x = e.clientX - rect.left
    let y = e.clientY - rect.top
    let bodyRect = this._getRelativeRect(this.refs.body)
    if (y >= bodyRect.top && y <= bodyRect.top+bodyRect.height) {
      // either on cell or on a row element
      let rowIdx = this._getRowIndex(y)
      let colIdx = this._getColumnIndex(x, 'strict')
      if (colIdx < 0) {
        return {
          type: 'row',
          rowIdx
        }
      } else {
        return {
          type: 'cell',
          rowIdx,
          colIdx
        }
      }
    } else {
      let headRect = this._getRelativeRect(this.refs.head)
      if (y >= headRect.top && y <= headRect.top+headRect.height) {
        // on column element
        let colIdx = this.getColumnIndex(x, 'strict')
        if (colIdx < 0) {
          return {
            type: 'corner'
          }
        } else {
          return {
            type: 'column',
            colIdx
          }
        }
      } else {
        let rowIdx = this._getRowIndex(y)
        let colIdx = this._getColumnIndex(x)
        return {
          type: 'outside',
          rowIdx, colIdx
        }
      }
    }
  }

  getRowIndex(clientY) {
    let rect = this._getRect()
    let y = clientY - rect.top
    return this._getRowIndex(y)
  }

  getColumnIndex(clientX) {
    let rect = this._getRect()
    let x = clientX - rect.left
    return this._getColumnIndex(x)
  }

  _getRect() {
    return getBoundingRect(this.el)
  }

  _getRelativeRect(comp) {
    let rect = this._getRect()
    let compRect = getBoundingRect(comp)
    compRect.top = compRect.top - rect.top
    compRect.left = compRect.left - rect.left
    return compRect
  }

  _getRowIndex(y) {
    const viewport = this._getViewport()
    let it = this.refs.body.el.getChildNodeIterator()
    let rowIdx = viewport.startRow
    while (it.hasNext()) {
      let rowEl = it.next()
      let rect = this._getRelativeRect(rowEl)
      if (y >= rect.top && y <= rect.top + rect.height) {
        return rowIdx
      }
      rowIdx++
    }
    return -1
  }

  _getColumnIndex(x) {
    const viewport = this._getViewport()
    let it = this.refs.head.el.getChildNodeIterator()
    // skip the first which is the corner element
    it.next()
    let colIdx = viewport.startCol
    while (it.hasNext()) {
      let cellEl = it.next()
      let rect = this._getRelativeRect(cellEl)
      if (x >= rect.left && x <= rect.left + rect.width) {
        return colIdx
      }
      colIdx++
    }
    return -1
  }

  // scrolling in a virtual grid of squares
  scroll(dx, dy) {
    // console.log('TableView.scroll()', dx, dy)
    let viewport = this._viewport
    viewport.dx += dx
    viewport.dy += dy
    // console.log('...', viewport.dx, viewport.dy)
    let dr = 0
    let dc = 0
    if (Math.abs(viewport.dy) > D) {
      dr = Math.round(viewport.dy / D)
      viewport.dy -= dr * D
      // console.log('... scrolling rows', dr)
    }
    if (Math.abs(viewport.dx) > D) {
      dc = Math.round(viewport.dx / D)
      viewport.dx -= dc * D
      // console.log('... scrolling cols', dc)
    }
    // stop if there is no change
    if (!dr && !dc) return

    const sheet = this._getSheet()
    const N = sheet.getRowCount()
    const M = sheet.getColumnCount()

    const oldStartRow = viewport.startRow
    const oldStartCol = viewport.startCol
    const newStartRow = Math.max(0, Math.min(N-1, oldStartRow+dr))
    const newStartCol = Math.max(0, Math.min(M-1, oldStartCol+dc))

    if (oldStartRow !== newStartRow || oldStartCol !== newStartCol) {
      viewport.startRow = newStartRow
      viewport.startCol = newStartCol
      this.rerender()
    }
  }

  scrollViewport(dr, dc) {
    const sheet = this._getSheet()
    const N = sheet.getRowCount()
    const M = sheet.getColumnCount()
    const viewport = this._getViewport()
    const oldStartRow = viewport.startRow
    const oldStartCol = viewport.startCol
    const newStartRow = Math.max(0, Math.min(N-1, oldStartRow+dr))
    const newStartCol = Math.max(0, Math.min(M-1, oldStartCol+dc))
    if (oldStartRow !== newStartRow || oldStartCol !== newStartCol) {
      viewport.startRow = newStartRow
      viewport.startCol = newStartCol
      this.rerender()
    }
  }

  getCell(rowIdx, colIdx) {
    const viewport = this._getViewport()
    const body = this.refs.body
    rowIdx = rowIdx - viewport.startRow
    colIdx = colIdx - viewport.startCol
    let tr = body.getChildAt(rowIdx)
    if (tr) {
      return tr.getChildAt(colIdx+1)
    }
  }

  _getCorner() {
    return this.refs.corner
  }

  _getSheet() {
    return this.props.sheet
  }

  _getWidth() {
    const width = this.props.width
    if (isNumber(width)) {
      return width
    } else if (isFunction(width)) {
      return width()
    } else {
      return 1000
    }
  }

  _getHeight() {
    const height = this.props.height
    if (isNumber(height)) {
      return height
    } else if (isFunction(height)) {
      return height()
    } else {
      return 750
    }
  }

  _getViewport() {
    return this._viewport
  }

}

function getBoundingRect(el) {
  let _rect = el.getNativeElement().getBoundingClientRect()
  return {
    top: _rect.top,
    left: _rect.left,
    height: _rect.height,
    width: _rect.width
  }
}
