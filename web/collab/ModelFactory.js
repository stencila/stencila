'use strict';

var oo = require('substance/util/oo');

var DocumentModel = require('../document/DocumentModel');

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

  this.create = function(schemaName) {
    if (schemaName === 'stencila-document') {
      return new DocumentModel();
    } else {
      throw new Error('Unhandled schema: '+ schemaName);
    }
  };

};

oo.initClass(ModelFactory);

module.exports = ModelFactory;
