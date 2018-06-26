import {Component} from 'substance'

const MAX_ROWS = 1000

export default
class TableValueComponent extends Component {
  render($$) {
    const table = this.props.value.data
    const data = table.data
    const columnNames = Object.keys(data)
    const cols = table.columns || columnNames.length
    const rows = table.rows || (cols > 0 && data[columnNames[0]] && data[columnNames[0]].length)

    let el = $$('div').addClass('sc-table-value')

    let tableEl = $$('table').addClass('sc-table-value')

    const thead = $$('thead')
    columnNames.forEach((name)=>{
      thead.append(
        $$('th').append(name)
      )
    })
    tableEl.append(thead)

    if (cols > 0) {
      const tbody = $$('tbody')
      for (let row = 0; row < rows && row < MAX_ROWS; row++) {
        let tr = $$('tr')
        columnNames.forEach((name)=>{
          tr.append(
            $$('td').text(data[name][row])
          )
        })
        tbody.append(tr)
      }
      if (this.props.pointer) {
        let tr = $$('tr')
        columnNames.forEach(()=>{
          tr.append(
            $$('td').text('...')
          )
        })
        tbody.append(tr)  
      }
      tableEl.append(tbody)
    }
    el.append(tableEl)

    if (rows > MAX_ROWS) {
      el.append(
        $$('div').addClass('se-more-records').append(
          `Showing ${MAX_ROWS} of ${rows} rows`
        )
      )
    }

    return el
  }
}
