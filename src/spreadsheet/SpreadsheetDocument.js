import { XMLDocument } from 'substance'
import SpreadsheetSchema from './SpreadsheetSchema'

export default class SpreadsheetDocument extends XMLDocument {

  getDocTypeParams() {
    return SpreadsheetSchema.getDocTypeParams()
  }

  getXMLSchema() {
    return SpreadsheetSchema
  }

  getRootNode() {
    return this.get('spreadsheet')
  }

  getName() {
    return this.getRootNode().find('name').text()
  }

  getCell(rowIdx, colIdx) {
    const data = this._getData()
    let row = data.getChildAt(rowIdx)
    let cell = row.getChildAt(colIdx)
    return cell
  }

  getValues(startRow, startCol, endRow, endCol) {
    let vals = []
    for (let rowIdx = startRow; rowIdx <= endRow; rowIdx++) {
      let rowVals = []
      for (let colIdx = startCol; colIdx <= endCol; colIdx++) {
        let cell = this.getCell(rowIdx, colIdx)
        rowVals.push(cell.textContent)
      }
      vals.push(rowVals)
    }
    return vals
  }

  setValues(startRow, startCol, vals) {
    for (let i = 0; i < vals.length; i++) {
      let row = vals[i]
      for (let j = 0; j < row.length; j++) {
        let val = row[j]
        let cell = this.getCell(startRow+i, startCol+j)
        if (cell) {
          cell.textContent = val
        }
      }
    }
  }

  clearRange(startRow, startCol, endRow, endCol) {
    for (let rowIdx = startRow; rowIdx <= endRow; rowIdx++) {
      for (let colIdx = startCol; colIdx <= endCol; colIdx++) {
        let cell = this.getCell(rowIdx, colIdx)
        cell.textContent = ''
      }
    }
  }

  getColumnCount() {
    const nrows = this.getRowCount()
    if (nrows > 0) {
      const data = this._getData()
      let firstRow = data.getFirstChild()
      return firstRow.getChildCount()
    } else {
      return 0
    }
  }

  getRowCount() {
    const data = this._getData()
    return data.getChildCount()
  }

  getColumnWidth(colIdx) { // eslint-disable-line
    // TODO: retrieve from model
    return 100
  }

  getRowHeight(rowIdx) { // eslint-disable-line
    // TODO: retrieve from model
    return 30
  }

  createRowsAt(rowIdx, n) {
    const M = this.getColumnCount()
    let data = this._getData()
    let rowAfter = data.getChildAt(rowIdx)
    for (let i = 0; i < n; i++) {
      let row = this.createElement('row')
      for (let j = 0; j < M; j++) {
        let cell = this.createElement('cell')
        // TODO: maybe insert default value?
        row.append(cell)
      }
      data.insertBefore(row, rowAfter)
    }
  }

  deleteRows(startRow, endRow) {
    let data = this._getData()
    for (let rowIdx = endRow; rowIdx >= startRow; rowIdx--) {
      let row = data.getChildAt(rowIdx)
      // TODO: add a helper to delete recursively
      row._childNodes.forEach((id) => {
        this.delete(id)
      })
      data.removeChild(row)
    }
  }

  createColumnsAt(colIdx, n) {
    // TODO: we need to add columns' meta, too
    // for each existing row insert new cells
    let data = this._getData()
    let it = data.getChildNodeIterator()
    while(it.hasNext()) {
      let row = it.next()
      let cellAfter = row.getChildAt(colIdx)
      for (let j = 0; j < n; j++) {
        let cell = this.createElement('cell')
        row.insertBefore(cell, cellAfter)
      }
    }
  }

  deleteColumns(startCol, endCol) {
    let data = this._getData()
    let N = this.getRowCount()
    for (let rowIdx = N-1; rowIdx >= 0; rowIdx--) {
      let row = data.getChildAt(rowIdx)
      for (let colIdx = endCol; colIdx >= startCol; colIdx--) {
        row.removeAt(colIdx)
      }
    }
  }

  fetchData(startRow, endRow) {
    return new Promise((resolve, reject)=>{
      resolve(true)
    })
  }

  _getData() {
    if (!this._dataNode) {
      this._dataNode = this.get('data')
    }
    return this._dataNode
  }

}
