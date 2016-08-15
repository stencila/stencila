'use strict';

var fs = require('fs');
var path = require('path');
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

  this.createDocument = function(schemaName) {
    if (schemaName === 'stencila-document') {
      return new DocumentModel();
    } else {
      throw new Error('Unhandled schema: '+ schemaName);
    }
  };

  this.readDocument = function(address, cb) {

    fs.readFile(path.join(address, 'index.html'), "utf8", function (err, content) {

      if(err) {
        return cb(new Err('ReadError', {
          cause: err
        }));
      }

      var importer = documentHTMLImporter;
      
      var doc = importer.importDocument(content);
      cb(null, doc);

    });
  };

  this.exportJson = function(doc) {
  	return documentJsonConverter.exportDocument(doc);
  };

};

oo.initClass(ModelFactory);

module.exports = ModelFactory;
