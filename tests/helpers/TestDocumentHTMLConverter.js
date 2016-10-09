import DocumentModel from '../../document/DocumentModel'

import TestHTMLImporter from './TestHTMLImporter'
import TestHTMLExporter from './TestHTMLExporter'

class TestDocumentHTMLConverter {

  constructor (config) {
    var converters = config.getConverterRegistry().get('html')
    this.importer = new TestHTMLImporter(
      DocumentModel,
      converters
    )
    this.exporter = new TestHTMLExporter(
      DocumentModel,
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
