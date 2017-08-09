import { registerSchema } from 'substance'
import SpreadsheetDocument from './SpreadsheetDocument'
import SpreadsheetSchema from './SpreadsheetSchema'

export default {
  name: 'Spreadsheet',

  configure(config) {
    // registers model nodes and a converter
    registerSchema(config, SpreadsheetSchema, SpreadsheetDocument)
  }
}
