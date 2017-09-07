import {
  Component,
} from 'substance'
import SpreadsheetCell from './SpreadsheetCell'

export default class SheetView extends Component {

  shouldRerender() {
    return false
  }

  didMount() {
    this._updateViewport()
    this.props.viewport.on('scroll', this._onScroll, this)
  }

  dispose() {
    this.props.viewport.off(this)
  }

  // rerender(...args) {
  //   console.log('### RENDERING sheet view')
  //   super.rerender(...args)
  // }

  render($$) {
    const sheet = this.props.sheet
    const viewport = this.props.viewport
    const M = sheet.getColumnCount()
    let el = $$('table').addClass('sc-table-view')
    let head = $$('tr').addClass('se-head').ref('head')
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

  _onScroll(dr, dc) {
    if (dc && !dr) {
      this._updateViewport()
    } else if (dr && !dc) {
      this.refs.body.update()
      this._updateBody()
    } else {
      this._updateHeader()
      this.refs.body.update()
      this._updateBody()
    }
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

const EPSILON = 0.1
function _epsilon(x) {
  if (Math.abs(x) < EPSILON) {
    return 0
  }
  return x
}