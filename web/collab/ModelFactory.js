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
var DocumentHTMLExporter = new DocumentHTMLExporter({ configurator: documentConfigurator });

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
   * Convert a Stencila component from HTML to JSON
   */
  this.convertDocument = function(schemaName, html, cb) {
    var importer;
    var exporter;
    if (schemaName === 'stencila-document') {
      importer = documentHTMLImporter;
      exporter = documentJsonConverter;
    } else {
      throw new Error('Unhandled schema: '+ schemaName);
    }

    var doc = importer.importDocument(html);
    var data = exporter.exportDocument(doc);
    cb(null, data);
  };


  /**
   * Read a Stencila component from a local file and convert to JSON
   * 
   * This method is probably only going to be used during development
   * to create a document from a test file
   */
  this.readDocument = function(schemaName, path, cb) {
    fs.readFile(path, "utf8", function (err, content) {
      if (err) return cb(new Err('ReadError', { message: err }));

      this.convertDocument(schemaName, content, function(err, data) {
        if (err) return cb(new Err('ConvertError', { message: err }));

        cb(null, data);
      });
    }.bind(this));
  };


};

oo.initClass(ModelFactory);

module.exports = ModelFactory;
