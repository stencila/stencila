export default function convertArrayToTable(array) {
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
    type: 'tab',
    data: tableData
  }
}
