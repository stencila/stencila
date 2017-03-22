import _multifunc from './_multifunc'

function table_array(array) {
  // TODO : check that array only has objects and that they have
  // consistent keys
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

function table_object(object) {
  // TODO : check that the object has only array properties and that
  // they are all of the same length
  return {
    type: 'table',
    data: object
  }
}

const table = _multifunc('table', {
  'table': table => table,
  'array': table_array,
  'object': table_object
})

export default table
