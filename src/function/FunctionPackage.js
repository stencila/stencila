import { registerSchema, BasePackage } from 'substance'

import FunctionDocument from './FunctionDocument'
import FunctionSchema from './FunctionSchema'
import FunctionDocumentImporter from './FunctionDocumentImporter'

export default {
  name: 'Function',

  configure(config) {
    registerSchema(config, FunctionSchema, FunctionDocument, {
      ImporterClass: FunctionDocumentImporter
    })

    config.import(BasePackage)
  }
}
