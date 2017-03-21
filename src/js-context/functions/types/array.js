import _multifunc from './_multifunc'

/**
 * Create an array of objects from a table
 *
 * @param  {[type]} table [description]
 * @return {[type]}       [description]
 */
function array_table(table) {
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

const array = _multifunc('array', {
  'table': array_table
})

export default array
