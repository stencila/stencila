'use strict';

var Document = require('substance/model/Document');

// Instantiate a configurator
var DocumentConfigurator = require('./DocumentConfigurator');
var configurator = new DocumentConfigurator();

/**
 * A Stencila Document data model
 *
 * @class      Document (name)
 * @param      {<type>}  schema  The schema
 */
var DocumentModel = function(schema) {
  DocumentModel.super.call(this, schema || DocumentModel.schema);

  this.user = 'joebloggs';
  this.rights = 'write';

  // Create a root body container node for the document
  this.create({
    type: "container",
    id: "body",
    nodes: []
  });
};

DocumentModel.Prototype = function() {

	this.refresh = function() {
		console.warn('TODO: refresh');
	}

	this.save = function() {
		// Instantiate an exporter (need to require here due to circular dependency)
		var DocumentHTMLExporter = require('./DocumentHTMLExporter');
		var htmlExporter = new DocumentHTMLExporter({
		  configurator: configurator
		});
		var content = htmlExporter.exportDocument(this);
		console.log(content);
	}

	this.commit = function() {
		console.warn('TODO: commit');
	}

};

DocumentModel.import = function(content) {
	// Instantiate an importer (need to require here due to circular dependency)
	var DocumentHTMLImporter = require('./DocumentHTMLImporter');
	var htmlImporter = new DocumentHTMLImporter({
	  configurator: configurator
	});
	return htmlImporter.importDocument(content);
};

Document.extend(DocumentModel);

module.exports = DocumentModel;
