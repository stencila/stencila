import {
  Component, isNumber, isFunction,
  platform, findParentDOMElement,
  DefaultDOMElement
} from 'substance'
import SpreadsheetCell from './SpreadsheetCell'

const D = 30
const P = 100

export default class TableView extends Component {

  getInitialState() {
    this._viewport = {
      // fictive scroll position: instead of real scroll
      // coordinates we apply a simple heuristic,
      // using a fixed height and width for every column
      // and a fictive position within this model
      x: 0,
      y: 0,
      // this is always the cell in the top-left corner
      startRow: 0,
      startCol: 0,
      // this is always the cell in the bottom-right corner
      // which is fully visible
      endRow: 0,
      endCol: 0,
    }
    return {}
  }

  shouldRerender() {
    return false
  }

  didMount() {
    this._updateViewport()
  }

  rerender(...args) {
    console.log('### RENDERING table view')
    super.rerender(...args)
  }

  render($$) {
    const sheet = this._getSheet()
    const N = sheet.getRowCount()
    const M = sheet.getColumnCount()
    const viewport = this._viewport

    let el = $$('table').addClass('sc-table-view')

    let head = $$('tr').ref('head')
    // HACK: 50px is currently the width of the label column
    // should be computed dynamically
    let width = 50
    head.append($$('th').addClass('se-corner').ref('corner'))
    for(let colIdx = 0; colIdx < M; colIdx++) {
      let w = sheet.getColumnWidth(colIdx)
      let th = $$('th').text(String(colIdx))
        .attr('data-col', colIdx)
        .css({ width: w })
      head.append(th)
      if (colIdx < viewport.startCol) {
        th.addClass('sm-hidden')
      } else {
        width += w
      }
    }
    el.append(head)
      .css({ width })

    let rowOffset = 0
    let pageIdx = 0
    while(rowOffset < N) {
      el.append($$(TablePage, {
        sheet,
        pageIdx,
        viewport
      }).ref(`p-${pageIdx}`))
      rowOffset += P
      pageIdx++
    }

    return el
  }

  getTargetForEvent(e) {
    let el = findParentDOMElement(e.target)
    return this._getTargetForElement(el)
  }

  _getTargetForElement(el) {
    while (el) {
      switch (el.tagName.toLowerCase()) {
        case 'td': {
          let rowIdx = parseInt(el.attr('data-row'), 10)
          let colIdx = parseInt(el.attr('data-col'), 10)
          return {
            type: 'cell',
            rowIdx,
            colIdx
          }
        }
        case 'th': {
          let rowIdx = el.attr('data-row')
          let colIdx = el.attr('data-col')
          if (rowIdx) {
            return {
              type: 'row',
              rowIdx: parseInt(rowIdx, 10)
            }
          } else if (colIdx) {
            return {
              type: 'column',
              colIdx: parseInt(colIdx, 10)
            }
          } else if (el.hasClass('se-corner')) {
            return {
              type: 'corner'
            }
          }
          break
        }
        default:
          //
      }
      if (el === this.el) break
      el = el.parentNode
    }
    return {
      type: 'outside'
    }
  }

  getRowIndex(clientY) {
    if (platform.inBrowser) {
      let _cornerRect = getBoundingRect(this.refs.corner.el)
      let clientX = _cornerRect.left
      let nativeEl = window.document.elementFromPoint(clientX, clientY)
      let el = DefaultDOMElement.wrap(nativeEl)
      let target = this._getTargetForElement(el)
      if (isNumber(target.rowIdx)) {
        return target.rowIdx
      }
    }
    return -1
  }

  getColumnIndex(clientX) {
    let rect = this._getRect()
    let x = clientX - rect.left
    return this._getColumnIndex(x)
  }

  _updateViewport() {
    let viewport = this._viewport
    let pageIdx = Math.floor(viewport.startRow / P)
    if (this._pageIdx !== pageIdx) {
      let old = {}
      if (isNumber(this._pageIdx)) {
        old[this._pageIdx] = true
        old[this._pageIdx+1] = true
      }
      delete old[pageIdx]
      delete old[pageIdx+1]
      let p1 = this._getPage(pageIdx)
      let p2 = this._getPage(pageIdx+1)
      if (p1) p1.show()
      if (p2) p2.show()
      Object.keys(old).forEach((pageIdx) => {
        let p = this._getPage(pageIdx)
        if (p) p.hide()
      })
      this._pageIdx = pageIdx
    }
    let page = this._getPage(this._pageIdx)

    viewport.H = this._getHeight()
    viewport.W = this._getWidth()
    viewport.height = this.refs.corner.el.getHeight()
    viewport.width = this.refs.corner.el.getWidth()
    viewport.endRow = viewport.startRow
    viewport.endCol = viewport.startCol

    let cols = this.refs.head.el.children
    let i
    for (i = 1; i < cols.length; i++) {
      let colIdx = i-1
      let th = cols[i]
      if (colIdx < viewport.startCol) {
        th.addClass('sm-hidden')
      } else {
        th.removeClass('sm-hidden')
        let w = th.getWidth()
        viewport.width += w
        if (viewport.width > viewport.W) break
        viewport.endCol++
      }
    }
    for (i = i+1; i < cols.length; i++) {
      let th = cols[i]
      th.addClass('sm-hidden')
    }
    this.el.css({ width: viewport.width })

    if (page) {
      page._updateVisibilities()
    }
    page = this._getPage(this._pageIdx+1)
    if (page) {
      page._updateVisibilities()
    }
  }

  _getPage(pageIdx) {
    return this.refs[`p-${pageIdx}`]
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
    const sheet = this._getSheet()
    const N = sheet.getRowCount()
    const M = sheet.getColumnCount()
    // console.log('TableView.scroll()', dx, dy)
    let viewport = this._viewport
    let oldX = viewport.x
    let oldY = viewport.y
    let oldC = Math.floor(oldX/D)
    let oldR = Math.floor(oldY/D)

    let newX = Math.max(0, Math.min(M*D, oldX+dx))
    let newY = Math.max(0, Math.min(N*D, oldY+dy))

    viewport.x = newX
    viewport.y = newY

    let newC = Math.floor(newX/D)
    let newR = Math.floor(newY/D)
    let dr = newR - oldR
    let dc = newC - oldC

    // stop if there is no change
    if (!dr && !dc) return

    const oldStartRow = viewport.startRow
    const oldStartCol = viewport.startCol
    const newStartRow = Math.max(0, Math.min(N-1, oldStartRow+dr))
    const newStartCol = Math.max(0, Math.min(M-1, oldStartCol+dc))

    if (oldStartRow !== newStartRow || oldStartCol !== newStartCol) {
      viewport.startRow = newStartRow
      viewport.startCol = newStartCol
      this._updateViewport()
    }
  }

  scrollViewport(dr, dc) {
    this.scroll(dr*D, dc*D)
  }

  getCellComponent(rowIdx, colIdx) {
    const sheet = this.props.sheet
    let cell = sheet.getCell(rowIdx, colIdx)
    let cellComp
    let page = this._getPage(this._pageIdx)
    if (page) {
      cellComp = page.refs[cell.id]
    }
    if (!cellComp) {
      page = this._getPage(this._pageIdx+1)
    }
    if (page) {
      cellComp = page.refs[cell.id]
    }
    // NOTE: we do not want the content, but
    // the surrounding <td> element here
    if (cellComp) {
      return cellComp.parent
    }
  }

  getCorner() {
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

class TablePage extends Component {

  getInitialState() {
    return 'hidden'
  }

  render($$) {
    let el = $$('tbody').addClass('sc-table-page')
    switch (this.state) {
      case 'loading': {
        el.addClass('sm-'+this.state)
        el.css({ height: P*D })
        break
      }
      case 'hidden': {
        el.addClass('sm-'+this.state)
        el.css({ height: P*D, display: 'none' })
        break
      }
      case 'visible': {
        let N = this.props.sheet.getRowCount()
        let rowOffset = this.props.pageIdx*P
        if (rowOffset<N) {
          let L = Math.min(rowOffset+P, N)
          for (let i = rowOffset; i < L; i++) {
            el.append(this._renderRow($$, i))
          }
        }
        break
      }
      default:
        throw new Error('Illegal state')
    }
    return el
  }

  _renderRow($$, rowIdx) {
    const sheet = this.props.sheet
    const viewport = this.props.viewport
    const M = sheet.getColumnCount()
    let tr = $$('tr').ref(String(rowIdx))
    tr.append(
      $$('th').text(String(rowIdx)).attr('data-row', rowIdx)
    )
    for (let j = 0; j < M; j++) {
      const cell = sheet.getCell(rowIdx, j)
      let td = $$('td')
        .append(
          $$(SpreadsheetCell, { node: cell }).ref(cell.id)
        ).attr({
          'data-row': rowIdx,
          'data-col': j
        })
      if (j < viewport.startCol || j > viewport.endCol) {
        td.addClass('sm-hidden')
      }
      tr.append(td)
    }
    return tr
  }

  show() {
    switch(this.state) {
      case 'hidden': {
        this._loadDataAndShow()
        break
      }
      case 'loading':
      case 'visible': {
        // nothing
        break
      }
      default:
        throw new Error('Illegal state')
    }
  }

  hide() {
    this.setState('hidden')
  }

  _loadDataAndShow() {
    const pageIdx = this.props.pageIdx
    let sheet = this.props.sheet
    let startRow = pageIdx*P
    let endRow = startRow+P-1
    this.setState('loading')
    sheet.fetchData(startRow, endRow).then(() => {
      if (this.state !== 'hidden') {
        this.setState('visible')
      }
    })
  }

  _updateVisibilities() {
    if (this.state !== 'visible') return

    const viewport = this.props.viewport
    const startRow = viewport.startRow
    const startCol = viewport.startCol
    const endCol = viewport.endCol
    const rowOffset = this.props.pageIdx*P
    let rows = this.el.children
    let i
    for (i = 0; i < rows.length; i++) {
      let tr = rows[i]
      let rowIdx = rowOffset+i
      if (rowIdx < startRow) {
        tr.addClass('sm-hidden')
      } else {
        tr.removeClass('sm-hidden')
        let cells = tr.children
        for (let j = 1; j < cells.length; j++) {
          let td = cells[j]
          let colIdx = j-1
          if (colIdx < startCol || colIdx > endCol) {
            td.addClass('sm-hidden')
          } else {
            td.removeClass('sm-hidden')
          }
        }
        let h = tr.getHeight()
        viewport.height += h
        viewport.endRow = rowIdx
        if (viewport.height > viewport.H) break
      }
    }
    for (i = i+1; i < rows.length; i++) {
      let tr = rows[i]
      tr.addClass('sm-hidden')
    }
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
