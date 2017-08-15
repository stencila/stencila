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
    if (!this.root) {
      this.root = this.find('spreadsheet')
    }
    return this.root
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

  _getData() {
    if (!this._dataNode) {
      this._dataNode = this.getRootNode().find('data')
    }
    return this._dataNode
  }
}
