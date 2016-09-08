'use strict';

var Document = require('substance/model/Document');

var DocumentConfigurator = require('./DocumentConfigurator');
var configurator = new DocumentConfigurator();

/**
 * A Stencila Document data model
 *
 * @class      Document (name)
 * @param      {<type>}  schema  The schema
 */
var DocumentModel = function (schema) {
  DocumentModel.super.call(this, schema || DocumentModel.schema);

  // Create a root body container node for the document
  this.create({
    type: 'container',
    id: 'content',
    nodes: []
  });
};

DocumentModel.Prototype = function () {
  this.execute = function (expression, context) {
    context = context || this.contexts[0];
    return context.execute(expression);
  };

  this.write = function (expression) {
    return this.contexts[0].write(expression);
  };
};

DocumentModel.schema = configurator.getSchema();

Document.extend(DocumentModel);

module.exports = DocumentModel;
