'use strict';

var HTMLImporter = require('substance/model/HTMLImporter');

var DocumentModel = require('./DocumentModel');

/**
 * Imports HTML into a Stencila Document
 *
 * @class      HTMLImporter (name)
 */
function DocumentHTMLImporter(options) {
  DocumentHTMLImporter.super.call(this, {
    // Required configuration for an importer
    DocumentClass: DocumentModel,
    schema: options.configurator.getSchema(),
    converters: options.configurator.getConverterRegistry().get('html'),
    // Override the default name for the id attribute
    idAttribute: 'data-uiid'
  });
}

DocumentHTMLImporter.Prototype = function() {

  /**
   * Convert HTML into a Stencila Document
   *
   * This method must be provided when extending HTMLImporter
   */
  this.convertDocument = function(els) {
    // The `containerId` argument should have the same
    // value as the `containerId` used by `ContainerEditor` in
    // `DocumentEditor`
    this.convertContainer(els, 'content');
  };

};

HTMLImporter.extend(DocumentHTMLImporter);


module.exports = DocumentHTMLImporter;
