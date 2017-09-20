import {Component} from 'substance'

const MAX_ROWS = 10

export default
class TableValueComponent extends Component {
  render($$) {
    let nrows
    const table = this.props.value
    let el = $$('div').addClass('sc-table-value')

    let tableEl = $$('table').addClass('sc-table-value')
    const columnNames = Object.keys(table.data)
    const thead = $$('thead')
    columnNames.forEach((name)=>{
      thead.append(
        $$('th').append(name)
      )
    })
    tableEl.append(thead)
    if (columnNames.length>0) {
      const tbody = $$('tbody')
      const data = table.data
      nrows = data[columnNames[0]].values.length

      for (let i = 0; i < nrows && i < MAX_ROWS; i++) {
        let tr = $$('tr')
        columnNames.forEach((name)=>{
          tr.append(
            $$('td').text(data[name].values[i])
          )
        })
        tbody.append(tr)
      }
      tableEl.append(tbody)
    }
    el.append(tableEl)

    if (nrows > MAX_ROWS) {
      el.append(
        $$('div').addClass('se-more-records').append(
          'Showing ',
          MAX_ROWS,
          ' out of ',
          nrows,
          ' rows'
        )
      )
    }

    return el
  }
}
