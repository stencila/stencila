import Datastore from './Datastore'

/**
 * A `Datastore` implemented in memory
 */
export default class MemoryDatastore extends Datastore {

  constructor(data) {
    super()

    this._data = data || {}
  }

  getColumnNames() {
    return Object.keys(this._data)
  }

  getRows(start, count) {
    const columnNames = this.getColumnNames()
    let numRows = this.getNumRows()
    if (start >= numRows || start < 0) {
      throw new Error('Index out of bounds')
    }
    let result = []
    let end = Math.min(start+count, numRows)
    for (let i = start; i < end; i++) {
      let row = {}
      columnNames.forEach((name) => {
        row[name] = this._data[name].values[i]
      })
      result.push(row)
    }
    return result
  }

  getNumRows() {
    let numRows = 0
    let cols = this.getColumnNames()
    if (cols.length > 0) {
      let firstCol = this._data[cols[0]]
      numRows = firstCol.values.length
    }
    return numRows
  }
}
