import { Component } from 'substance'
import SpreadsheetLayout from './SpreadsheetLayout'
import SpreadsheetCell from './SpreadsheetCell'

export default class SpreadsheetComponent extends Component {

  getInitialState() {
    this._layout = new SpreadsheetLayout(this.props.sheet)
    const viewPort = this._layout.getViewport(0, 0)
    return Object.assign({}, viewPort)
  }

  render($$) {
    let el = $$('div').addClass('sc-spreadsheet')
    el.append(
      $$('div').addClass('se-content').append(
        this._renderTable($$)
      )
    )
    el.on('wheel', this._onWheel)
    return el
  }

  _renderTable($$) {
    // TODO: on the long term we want to render a view-port only,
    // not the whole table
    const sheet = this.props.sheet
    const data = sheet.find('data')
    let table = $$('table').css({
      width: this._layout.getWidth(this.state.startCol, this.state.endCol)
    })
    // TODO: some convenience for retrieving sheet stats would be nice
    let rows = data.children
    let nrows = rows.length
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
    let tr = $$('tr')
    tr.append($$('th'))
    for (let i = this.state.startCol; i < this.state.endCol; i++) {
      // TODO: map to ABC etc...
      tr.append($$('th').append(String(i+1)))
    }
    head.append(tr)
    return head
  }

  _renderBody($$) {
    const sheet = this.props.sheet
    const data = sheet.find('data')
    const rows = data.children
    let body = $$('tbody')
    for (let i = this.state.startRow; i < this.state.endRow; i++) {
      const row = rows[i]
      let tr = $$('tr').ref(String(i))
      tr.append($$('th').text(String(i)))
      let cells = row.children
      for (let j = this.state.startCol; j < this.state.endCol; j++) {
        tr.append(
          $$('td').append(
            $$(SpreadsheetCell, { node:cells[j] })
          )
        )
      }
      body.append(tr)
    }
    return body
  }

  _onWheel(e) {
    e.stopPropagation()
    e.preventDefault()
    console.log('le wheeeeeel', e)
    let deltaX = _sign(e.deltaX)
    let deltaY = _sign(e.deltaY)
    if (deltaX || deltaY) {
      let newStartCol = this.state.startCol + deltaX
      let newStartRow = this.state.startRow + deltaY
      this.extendState(this._layout.getViewport(newStartRow, newStartCol))
    }
  }

}

// signum with epsilon
const EPSILON = 1
function _sign(x) {
  let abs = Math.abs(x)
  if (abs > EPSILON) {
    return Math.sign(x)
  }
  return 0
}
