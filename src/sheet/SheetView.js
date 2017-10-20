import {
  Component, getRelativeBoundingRect, RenderingEngine
} from 'substance'
import SheetColumnHeader from './SheetColumnHeader'
import SheetRowHeader from './SheetRowHeader'
import SheetCell from './SheetCell'
import getBoundingRect from '../util/getBoundingRect'

export default class SheetView extends Component {

  shouldRerender(newProps) {
    if(newProps.mode !== this.props.mode) return true
    return false
  }

  didMount() {
    this._updateViewport()
    this.props.viewport.on('scroll', this._onScroll, this)
  }

  didUpdate() {
    this._updateViewport()
  }

  dispose() {
    this.props.viewport.off(this)
  }

  update() {
    this.rerender()
  }

  render($$) {
    const sheet = this.props.sheet
    const mode = this.props.mode
    const viewport = this.props.viewport
    const M = sheet.getColumnCount()

    let el = $$('table').addClass('sc-table-view sm-mode-' + mode)
    let head = $$('tr').addClass('se-head').ref('head')
    let corner = $$('th').addClass('se-corner').ref('corner')

    // ATTENTION: we have a slight problem here.
    // <table> with fixed layout needs the exact width
    // so that the column widths are correct.
    // To avoid that corrupting the layout we need
    // to make sure to set the correct value here
    // Unfortunately this means that we must set the corner width here
    corner.css({ width: 50 })
    let width = 50
    head.append(corner)
    for(let colIdx = 0; colIdx < M; colIdx++) {
      let columnMeta = sheet.getColumnMeta(colIdx)
      let th = $$(SheetColumnHeader, { node: columnMeta, colIdx }).ref(columnMeta.id)
      let w = th.getWidth()
      if (colIdx < viewport.startCol) {
        th.addClass('sm-hidden')
      } else {
        width += w
      }
      head.append(th)
    }
    el.css({ width })
    el.append(head)
    el.append(
      $$(TableBody, { sheet, viewport }).ref('body')
    )
    return el
  }

  _updateViewport() {
    this._updateHeader()
    this._updateBody()
  }

  _updateHeader() {
    let viewport = this.props.viewport
    // Note: in contrast to the render method
    // we can use the real width here
    viewport.width = this.refs.corner.el.getWidth()
    viewport.endCol = viewport.startCol

    const W = viewport.getContainerWidth()

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
        if (viewport.width > W) {
          break
        }
        viewport.endCol++
      }
    }
    for (i = i+1; i < cols.length; i++) {
      let th = cols[i]
      th.addClass('sm-hidden')
    }
    this.el.css({ width: viewport.width })
  }

  _updateBody() {
    let viewport = this.props.viewport
    viewport.height = this.refs.corner.el.getHeight()
    viewport.endRow = viewport.startRow

    const H = viewport.getContainerHeight()

    // show only cells which are inside the viewport
    let rowIt = this.refs.body.el.getChildNodeIterator()
    let rowIdx = viewport.startRow
    while (rowIt.hasNext()) {
      let row = rowIt.next()
      let cols = row.children
      for (let i = 1; i < cols.length; i++) {
        let td = cols[i]
        let colIdx = i-1
        if (colIdx < viewport.startCol || colIdx > viewport.endCol) {
          td.addClass('sm-hidden')
        } else {
          td.removeClass('sm-hidden')
        }
      }
      let h = row.getHeight()
      viewport.height += h
      if (viewport.height < H) {
        viewport.endRow = rowIdx
      }
      rowIdx++
    }
  }

  getBoundingRect(rowIdx, colIdx) {
    let top = 0, left = 0, height = 0, width = 0
    // in header
    let rowComp
    if (rowIdx === -1) {
      rowComp = this.refs.head
    } else {
      rowComp = this.refs.body.getRowComponent(rowIdx)
    }
    if (rowComp) {
      let rect = getRelativeBoundingRect(rowComp.el, this.el)
      top = rect.top
      height = rect.height
    }
    let colComp
    if (colIdx === -1) {
      colComp = this.refs.corner
    } else {
      colComp = this.refs.head.getChildAt(colIdx+1)
    }
    if (colComp) {
      let rect = getRelativeBoundingRect(colComp.el, this.el)
      left = rect.left
      width = rect.width
    }
    return { top, left, width, height }
  }

  getCellComponent(rowIdx, colIdx) {
    if (rowIdx === -1) {
      // retrieve a header cell
      return this.refs.head.getChildAt(colIdx+1)
    } else {
      let tr = this.refs.body.getRowComponent(rowIdx)
      if (tr) {
        return tr.getCellComponent(colIdx)
      }
    }
    // otherwise
    return null
  }

  getCellComponentForCell(cell) {
    // TODO: need to revisit this for a better implementation
    return this.refs.body.find(`td[data-cell-id="${cell.id}"]`)
  }

  getCornerComponent() {
    return this.refs.corner
  }

  /*
   * Tries to resolve row and column index, and type of cell
   * for a given event
   */
  getTargetForEvent(e) {
    const clientX = e.clientX
    const clientY = e.clientY
    let colIdx = this.getColumnIndex(clientX)
    let rowIdx = this.getRowIndex(clientY)
    let type
    if (colIdx >= 0 && rowIdx >= 0) {
      type = 'cell'
    } else if (colIdx === -1 && rowIdx >= 0) {
      type = 'row'
    } else if (colIdx >= 0 && rowIdx === -1) {
      type = 'column'
    } else if (colIdx === -1 && rowIdx === -1) {
      type = 'corner'
    } else {
      type = 'outside'
    }
    return { type, rowIdx, colIdx }
  }

  getColumnIndex(x) {
    const headEl = this.refs.head.el
    const children = headEl.children
    for (let i = 0; i < children.length; i++) {
      let child = children[i]
      if (_isXInside(x, getBoundingRect(child))) {
        return i-1
      }
    }
    return undefined
  }

  getRowIndex(y) {
    const headEl = this.refs.head.el
    if (_isYInside(y, getBoundingRect(headEl))) {
      return -1
    }
    const bodyEl = this.refs.body.el
    const children = bodyEl.children
    for (let i = 0; i < children.length; i++) {
      let child = children[i]
      if (_isYInside(y, getBoundingRect(child))) {
        return parseInt(child.getAttribute('data-row'), 10)
      }
    }
    return undefined
  }

  _onScroll(dr, dc) {
    if (dc && !dr) {
      this._updateViewport()
    } else if (dr && !dc) {
      this.refs.body.update()
      this._updateViewport()
    } else {
      this.refs.body.update()
      this._updateViewport()
    }
  }

}

function _isXInside(x, rect) {
  return x >= rect.left && x <= rect.left+rect.width
}

function _isYInside(y, rect) {
  return y >= rect.top && y <= rect.top+rect.height
}

class TableBody extends Component {

  getInitialState() {
    return {}
  }

  render($$) {
    let el = $$('tbody')
    let sheet = this.props.sheet
    let viewport = this.props.viewport
    let N = sheet.getRowCount()
    let n = Math.min(viewport.startRow+viewport.P, N)
    for (let i = viewport.startRow; i < n; i++) {
      el.append(
        $$(TableRow, {
          sheet, viewport,
          rowIdx: i
        }).ref(String(i))
      )
    }
    this._startRow = viewport.startRow
    this._nextRow = n
    return el
  }

  /*
    TODO: this could be speeded-up by incrementally
    adding rows and cols instead of relying on reactive rendering.
  */
  update() {
    const viewport = this.props.viewport
    let dr = viewport.startRow - this._startRow
    if (dr > 0 && dr < viewport.P) {
      this._append(dr)
    } else if (dr < 0 && dr > -viewport.P) {
      this._prepend(dr)
    } else {
      this.rerender()
    }
  }

  getRowComponent(rowIdx) {
    return this.refs[rowIdx]
  }

  _append(dr) {
    let renderContext = RenderingEngine.createContext(this)
    let $$ = renderContext.$$
    let sheet = this.props.sheet
    let viewport = this.props.viewport
    const N = sheet.getRowCount()
    for(let i=0; i<dr; i++) {
      // Note: to be able to scroll to the very end
      // we stop appending rows when hitting the bottom of the sheet
      // but still removing the first row
      let rowIndex = this._nextRow++
      if (rowIndex < N) {
        this.append(
          $$(TableRow, {
            sheet, viewport,
            rowIdx: rowIndex
          }).ref(String(rowIndex))
        )
      }
      this.removeChild(this.getChildAt(0))
      this._startRow++
    }
  }

  _prepend(dr) {
    let renderContext = RenderingEngine.createContext(this)
    let $$ = renderContext.$$
    let sheet = this.props.sheet
    let viewport = this.props.viewport
    for(let i=0; i>dr; i--) {
      this._startRow--
      let rowIndex = this._startRow
      this.insertAt(0,
        $$(TableRow, {
          sheet, viewport,
          rowIdx: rowIndex
        }).ref(String(rowIndex))
      )
      // only remove the trailing row if there are enough
      // rows present already
      if (this.getChildCount() > viewport.P) {
        this.removeChild(this.getChildAt(this.getChildCount()-1))
      }
      this._nextRow--
    }
  }

}

class TableRow extends Component {

  getInitialState() {
    return 'loading'
  }

  didMount() {
    this._loadDataAndShow()
  }

  render($$) {
    let sheet = this.props.sheet
    let rowIdx = this.props.rowIdx
    let viewport = this.props.viewport
    let height = sheet.getRowHeight(rowIdx)
    let el = $$('tr')
    switch (this.state) {
      case 'hidden':
      case 'loading': {
        el.addClass('sm-'+this.state)
        break
      }
      case 'visible': {
        let M = sheet.getColumnCount()
        let row = sheet.get('row-' + (rowIdx + 1))
        el.append(
          $$(SheetRowHeader, {node: row, rowIdx: rowIdx}).ref(String(-1))
        )
        for (let j = 0; j < M; j++) {
          const cell = sheet.getCell(rowIdx, j)
          let td = $$('td').ref(String(j))
            .append(
              $$(SheetCell, { node: cell }).ref(cell.id)
            ).attr({
              'data-row': rowIdx,
              'data-col': j,
              'data-cell-id': cell.id
            })
          if (j < viewport.startCol || j > viewport.endCol) {
            td.addClass('sm-hidden')
          }

          el.append(td)
        }
        break
      }
      default:
        throw new Error('Illegal state')
    }
    el.attr('data-row', rowIdx)
    el.css({
      "height": height
    })
    return el
  }

  hide() {
    this.setState('hidden')
  }

  getCellComponent(colIdx) {
    return this.refs[colIdx]
  }

  _loadDataAndShow() {
    const sheet = this.props.sheet
    const rowIdx = this.props.rowIdx
    this.setState('loading')
    sheet.ensureRowAvailable(rowIdx).then(() => {
      if (this.state !== 'hidden') {
        this.setState('visible')
      }
    })
  }
}
