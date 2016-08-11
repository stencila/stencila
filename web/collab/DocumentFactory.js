'use strict';

var oo = require('substance/util/oo');

var DocumentModel = require('../document/DocumentModel');

/**
 * A factory for creating Stencila components based on the
 * schema name
 * 
 * Used by `./SnapshotEngine`. The "Document" in `DocumentFactory`
 * refers to a Substance `Document` and this factor can create
 * all types of Stencila components
 *
 * @class      DocumentFactory (name)
 * @param      {<type>}  config  The configuration
 */
function DocumentFactory(config) {
}

DocumentFactory.Prototype = function() {

  this.create = function(schemaName) {
    if (schemaName === 'stencila-document') {
      return new DocumentModel();
    } else {
      throw new Error('Unhandled schema: '+ schemaName);
    }
  };

};

oo.initClass(DocumentFactory);

module.exports = DocumentFactory;
