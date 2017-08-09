import { XMLSchema } from 'substance'
import SpreadsheetSchemaData from '../../tmp/SpreadsheetSchema.data'

const SpreadsheetSchema = XMLSchema.fromJSON(SpreadsheetSchemaData)

// TODO: this should come from compilation
SpreadsheetSchema.getName = function() {
  return 'stencila-spreadsheet'
}

SpreadsheetSchema.getVersion = function() {
  return '1.0'
}

SpreadsheetSchema.getDocTypeParams = function() {
  return ['spreadsheet', 'Stencila Spreadsheet 1.0', SpreadsheetSchema.uri]
}

SpreadsheetSchema.uri = 'http://stenci.la/Spreadsheet-1.0.dtd'

export default SpreadsheetSchema
