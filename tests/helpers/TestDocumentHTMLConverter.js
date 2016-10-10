import DocumentModel from '../../document/DocumentModel'

import TestHTMLImporter from './TestHTMLImporter'
import TestHTMLExporter from './TestHTMLExporter'

class TestDocumentHTMLConverter {

  constructor (config) {
    let converters = config.getConverterRegistry().get('html')
    let schema = config.getSchema()
    this.importer = new TestHTMLImporter(
      DocumentModel,
      schema,
      converters
    )
    this.exporter = new TestHTMLExporter(
      DocumentModel,
      schema,
      converters
    )
  }

  import (html) {
    return this.importer.importDocument(html)
  }

  export (doc) {
    return this.exporter.exportDocument(doc)
  }
}

export default TestDocumentHTMLConverter
