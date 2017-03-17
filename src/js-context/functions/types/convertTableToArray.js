export default function convertTableToArray(table) {
  let colNames = Object.keys(table.data)
  let rows = table.data[colNames[0]].length
  let array = []
  for (var rowNumber = 0; rowNumber < rows; rowNumber++) {
    let rowData = {}
    for (let colName of colNames) {
      rowData[colName] = table.data[colName][rowNumber]
    }
    array.push(rowData)
  }
  return array
}
