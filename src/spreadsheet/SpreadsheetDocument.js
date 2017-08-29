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

  _getData() {
    if (!this._dataNode) {
      this._dataNode = this.get('data')
    }
    return this._dataNode
  }

}
