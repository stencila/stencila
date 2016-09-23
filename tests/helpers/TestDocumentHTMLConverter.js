import oo from 'substance/util/oo'

import DocumentModel from '../../document/DocumentModel'

import TestHTMLImporter from './TestHTMLImporter'
import TestHTMLExporter from './TestHTMLExporter'

function TestDocumentHTMLConverter (config) {
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

TestDocumentHTMLConverter.Prototype = function () {
  this.import = function (html) {
    return this.importer.importDocument(html)
  }

  this.export = function (doc) {
    return this.exporter.exportDocument(doc)
  }
}

oo.initClass(TestDocumentHTMLConverter)

module.exports = TestDocumentHTMLConverter
