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

}
