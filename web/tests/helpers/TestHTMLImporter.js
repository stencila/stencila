'use strict';

var HTMLImporter = require('substance/model/HTMLImporter');

function TestHTMLImporter(DocumentClass, converters) {
  TestHTMLImporter.super.call(this, {
    DocumentClass: DocumentClass,
    schema: DocumentClass.schema,
    converters: converters
  });
}

TestHTMLImporter.Prototype = function() {

  this.convertDocument = function(els) {
    this.convertContainer(els, 'content');
  };

};

HTMLImporter.extend(TestHTMLImporter);

module.exports = TestHTMLImporter;
