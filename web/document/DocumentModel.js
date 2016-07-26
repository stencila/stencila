'use strict';

var Document = require('substance/model/Document');

/**
 * A Stencila Document data model
 *
 * @class      Document (name)
 * @param      {<type>}  schema  The schema
 */
var DocumentModel = function(schema) {
  DocumentModel.super.call(this, schema || DocumentModel.schema);

  this.rights = 'write';

  // Create a root body container node for the document
  this.create({
    type: "container",
    id: "body",
    nodes: []
  });
};

DocumentModel.Prototype = function() {
};

Document.extend(DocumentModel);

module.exports = DocumentModel;
