'use strict';

var fs = require('fs');
var path = require('path');

var Err = require('substance/util/SubstanceError');
var oo = require('substance/util/oo');

var DocumentModel = require('../document/DocumentModel');

var DocumentConfigurator = require('../document/DocumentConfigurator');
var documentConfigurator = new DocumentConfigurator();

var DocumentHTMLImporter = require('../document/DocumentHTMLImporter');
var documentHTMLImporter = new DocumentHTMLImporter({ configurator: documentConfigurator });

var DocumentHTMLExporter = require('../document/DocumentHTMLExporter');
var documentHTMLExporter = new DocumentHTMLExporter({ configurator: documentConfigurator });

var DocumentJSONConverter = require('../document/DocumentJSONConverter');
var documentJsonConverter = new DocumentJSONConverter();

/**
 * A factory for creating Stencila component models based on the
 * schema name
 * 
 * Used by `./SnapshotEngine`.
 *
 * @class      ModelFactory (name)
 * @param      {<type>}  config  The configuration
 */
function ModelFactory(config) {
}

ModelFactory.Prototype = function() {

  /**
   * Create a new, empty Stencila component from the `schemaName`
   *
   * @param      {string}         schemaName  The schema name
   */
  this.createDocument = function(schemaName) {
    if (schemaName === 'stencila-document') {
      return new DocumentModel();
    } else {
      throw new Error('Unhandled schema: '+ schemaName);
    }
  };

  /**
   * Import a Stencila component from HTML to JSON
   */
  this.importDocument = function(schemaName, format, content, cb) {
    if (format !== 'html') throw new Error('Unhandled format: '+ format);

    var importer;
    var exporter;
    if (schemaName === 'stencila-document') {
      importer = documentHTMLImporter;
      exporter = documentJsonConverter;
    } else {
      throw new Error('Unhandled schema: '+ schemaName);
    }

    // Force importer to create a new document. See https://github.com/substance/substance/issues/765
    importer.createDocument();
    var doc = importer.importDocument(content);
    var data = exporter.exportDocument(doc);
    cb(null, data);
  };

  /**
   * Export a Stencila component frm JSON to HTML
   */
  this.exportDocument = function(schemaName, format, content, cb) {
    if (format !== 'html') throw new Error('Unhandled format: '+ format);

    var importer;
    var exporter;
    if (schemaName === 'stencila-document') {
      importer = documentJsonConverter;
      exporter = documentHTMLExporter;
    } else {
      throw new Error('Unhandled schema: '+ schemaName);
    }

    var doc = this.createDocument(schemaName);
    importer.importDocument(doc, content);
    var data = exporter.exportDocument(doc);

    // Remove "data-id" attributes
    data = data.replace(/ data-id=\".+?\"/g, '');

    cb(null, data);
  };

};

oo.initClass(ModelFactory);

module.exports = ModelFactory;
