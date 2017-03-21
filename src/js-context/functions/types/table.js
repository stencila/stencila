import _multifunc from './_multifunc'

function table_array(array) {
  let tableData = {}
  array.forEach(row => {
    Object.keys(row).forEach(key => {
      if (tableData[key]) {
        tableData[key].push(row[key])
      } else {
        tableData[key] = [row[key]]
      }
    })
  })
  return {
    type: 'table',
    data: tableData
  }
}

const table = _multifunc('table', {
  'array': table_array
})

export default table
