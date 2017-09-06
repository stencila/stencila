import {
  Component, isNumber, isFunction
} from 'substance'
import SpreadsheetCell from './SpreadsheetCell'

export default class SheetView extends Component {

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
    const sheet = this.props.sheet
    const viewport = this.props.viewport
    const M = sheet.getColumnCount()
    let el = $$('table').addClass('sc-table-view')
    let head = $$('tr').ref('head')
    let corner = $$('th').addClass('se-corner').ref('corner')
      .css({ width: 50})
    let width = 50
    head.append(corner)
    for(let colIdx = 0; colIdx < M; colIdx++) {
      let w = sheet.getColumnWidth(colIdx)
      let th = $$('th').text(String(colIdx))
        .attr('data-col', colIdx)
        .css({ width: w })
      width += w
      head.append(th)
    }
    el.css({ width })
    el.append(head)
    el.append(
      $$(TableBody, { sheet, viewport }).ref('body')
    )
    return el
  }

  _updateHeader() {
    let viewport = this.props.viewport
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

  _updateViewport() {
    this._updateHeader()
    this._updateBody()
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

  getTargetForEvent() {
    // TODO
    return { type: 'outside '}
  }

  getRowIndex() {
    return -1
  }

  getColumnIndex() {
    return -1
  }

  getCellComponent() {
    return null
  }

  getCorner() {
    return this.refs.corner
  }

  // scrolling in a virtual grid of squares
  scroll(dx, dy) {
    const sheet = this.props.sheet
    const N = sheet.getRowCount()
    const M = sheet.getColumnCount()
    // console.log('TableView.scroll()', dx, dy)
    let viewport = this.props.viewport
    let oldX = viewport.x
    let oldY = viewport.y
    let oldC = Math.floor(oldX/viewport.D)
    let oldR = Math.floor(oldY/viewport.D)
    let newX = Math.max(0, Math.min(M*viewport.D, oldX+dx))
    let newY = Math.max(0, Math.min(N*viewport.D, oldY+dy))
    viewport.x = newX
    viewport.y = newY
    let newC = Math.floor(newX/viewport.D)
    let newR = Math.floor(newY/viewport.D)
    let dr = newR - oldR
    let dc = newC - oldC
    // stop if there is no change
    if (!dr && !dc) return
    const oldStartRow = viewport.startRow
    const oldStartCol = viewport.startCol
    const newStartRow = Math.max(0, Math.min(N-1, oldStartRow+dr))
    const newStartCol = Math.max(0, Math.min(M-1, oldStartCol+dc))

    if (oldStartRow === newStartRow && oldStartCol !== newStartCol) {
      viewport.startCol = newStartCol
      this._updateViewport()
    } else if (oldStartRow !== newStartRow && oldStartCol === newStartCol) {
      viewport.startRow = newStartRow
      this.refs.body.update()
      this._updateBody()
    } else {
      console.assert(false, 'Thought that it was not possible to scroll x and y at the same time.')
    }
  }

  scrollViewport(dr, dc) {
    const viewport = this.props.viewport
    this.scroll(dr*viewport.D, dc*viewport.D)
  }

}

class TableBody extends Component {

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
    return el
  }

  /*
    TODO: we could optimize this for optimization
    i.e.
  */
  update() {
    this.rerender()
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
        el.append(
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

// function getBoundingRect(el) {
//   let _rect = el.getNativeElement().getBoundingClientRect()
//   return {
//     top: _rect.top,
//     left: _rect.left,
//     height: _rect.height,
//     width: _rect.width
//   }
// }
