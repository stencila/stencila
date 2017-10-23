import { XMLSchema } from 'substance'
import SheetSchemaData from '../../tmp/SheetSchema.data'

const SheetSchema = XMLSchema.fromJSON(SheetSchemaData)

// TODO: this should come from compilation
SheetSchema.getName = function() {
  return 'stencila-sheet'
}

SheetSchema.getVersion = function() {
  return '1.0'
}

SheetSchema.getDocTypeParams = function() {
  return ['sheet', 'Stencila Sheet 1.0', SheetSchema.uri]
}

SheetSchema.getDefaultTextType = function () {
  return 'cell'
}

SheetSchema.uri = 'http://stenci.la/Sheet-1.0.dtd'

export default SheetSchema
