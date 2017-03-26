import _multifunc from './_multifunc'

/**
 * Create a table from an array
 */
function table_array(array) {
  // TODO : check that array only has objects and that they have
  // consistent keys
  let data = {}

  let fields = Object.keys(array[0])
  for (let field of fields) {
    data[field] = {
      values: []
    }
  }

  for (let row of array) {
    for (let field of fields) {
      data[field].values.push(row[field])
    }
  }

  return {
    type: 'table',
    data: data
  }
}

/**
 * Create a table from an object
 */
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
