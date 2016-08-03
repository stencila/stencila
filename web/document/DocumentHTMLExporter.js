'use strict';

var HTMLExporter = require('substance/model/HTMLExporter');

var DocumentModel = require('./DocumentModel');

/**
 * Exports a Stencila Document to HTML
 *
 * @class      HTMLExporter (name)
 */
function DocumentHTMLExporter(options) {
  DocumentHTMLExporter.super.call(this, {
    converters: options.configurator.getConverterRegistry().get('html'),
    // Override the default name for the id attribute
    idAttribute: 'data-uiid'
  });
}

DocumentHTMLExporter.Prototype = function() {

  /**
   * Export a Stencila Document to HTML
   */
  this.exportDocument = function(doc) {
    var bodyNodes = this.convertContainer(doc.get('content'));
    var wrapper = this.$$("div").append(bodyNodes);
    return wrapper.html();
  };

};

HTMLExporter.extend(DocumentHTMLExporter);


module.exports = DocumentHTMLExporter;
