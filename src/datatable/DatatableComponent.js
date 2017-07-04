import { Component, forEach } from 'substance'

export default class DatatableComponent extends Component {

  getInitialState() {
    return {
      // TODO: discuss naming and approach
      start: 0,
      count: 100
    }
  }

  render($$) {
    const doc = this.props.document
    const store = doc.getStore()
    const { start, count } = this.state

    let el = $$('table').addClass('sc-datatable')

    // render the column names
    let header = $$('thead').addClass('se-header')
    let columnNames = store.getColumnNames()
    columnNames.forEach((name) => {
      header.append($$('th').text(name))
    })
    el.append(header)

    const rows = store.getRows(start, count)
    const body = $$('tbody').addClass('se-body')
    body.append(
      rows.map((row, idx) => {
        return this._renderRow($$, row, idx)
      })
    )
    el.append(body)

    return el
  }

  _renderRow($$, row, idx) {
    let el = $$('tr').addClass('se-row').ref(String(idx))
    forEach(row, (val, name) => {
      el.append(
        // TODO: is it enough to just render the value as string?
        $$('td').text(val).ref(`${idx}.${name}`)
      )
    })
    return el
  }

}