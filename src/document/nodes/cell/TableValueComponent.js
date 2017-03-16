import {Component} from 'substance'

export default
class TableValueComponent extends Component {
  render($$) {
    const table = this.props.value
    let el = $$('table').addClass('sc-table-value')
    const columnNames = Object.keys(table.data)
    const thead = $$('thead')
    columnNames.forEach((name)=>{
      thead.append(
        $$('th').append(name)
      )
    })
    el.append(thead)
    if (columnNames.length>0) {
      const tbody = $$('tbody')
      const data = table.data
      const nrows = data[columnNames[0]].length
      for (let i = 0; i < nrows; i++) {
        let tr = $$('tr')
        columnNames.forEach((name)=>{
          tr.append(
            $$('td').text(data[name][i])
          )
        })
        tbody.append(tr)
      }
      el.append(tbody)
    }
    return el
  }
}