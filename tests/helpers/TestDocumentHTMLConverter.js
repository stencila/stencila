import oo from 'substance/util/oo'

var DocumentModel = require('../../document/DocumentModel');

var TestHTMLImporter = require('./TestHTMLImporter');
var TestHTMLExporter = require('./TestHTMLExporter');

function TestDocumentHTMLConverter (config) {
  var converters = config.getConverterRegistry().get('html');
  this.importer = new TestHTMLImporter(
    DocumentModel,
    converters
  );
  this.exporter = new TestHTMLExporter(
    DocumentModel,
    converters
  );
}

TestDocumentHTMLConverter.Prototype = function () {
  this.import = function (html) {
    return this.importer.importDocument(html);
  };

  this.export = function (doc) {
    return this.exporter.exportDocument(doc);
  };
};

oo.initClass(TestDocumentHTMLConverter);

module.exports = TestDocumentHTMLConverter;
