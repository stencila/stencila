import { XMLSchema } from 'substance'

import FunctionSchemaData from '../../tmp/FunctionSchema.data'

const FunctionSchema = XMLSchema.fromJSON(FunctionSchemaData)

FunctionSchema.getName = function() {
  return 'stencila-function'
}

FunctionSchema.getVersion = function() {
  return '1.0'
}

FunctionSchema.getDocTypeParams = function() {
  return ['function', 'Stencila Function 1.0', FunctionSchema.uri]
}

FunctionSchema.uri = 'http://stenci.la/Function-1.0.dtd'

export default FunctionSchema
