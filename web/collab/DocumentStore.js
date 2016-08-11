'use strict';

var DocumentStoreBase = require('substance/collab/DocumentStore');
var extend = require('lodash/object/extend');

/**
 * Stores Stencila component sessions.
 * 
 * Used to sync component sessions across collaborators.
 *
 * @class      DocumentStore (name)
 * @param      {<type>}  config  The configuration
 */
function DocumentStore(config) {
  DocumentStore.super.apply(this, arguments);

  this._documents = {
    
    'default@edit' : {
      id: 'default@edit',
      schemaName: 'stencila-document',
      version: 0
    }
    
  };
}

DocumentStore.Prototype = function() {

  this._createDocument = function(props) {
    console.log('DocumentStore._createDocument' + JSON.stringify(props));
    this._documents[props.documentId] = props;
  };

  this._deleteDocument = function(documentId) {
    console.log('DocumentStore._deleteDocument ' + documentId);
    delete this._documents[documentId];
  };

  this._getDocument = function(documentId) {
    console.log('DocumentStore._getDocument ' + documentId);
    return this._documents[documentId];
  };

  this._updateDocument = function(documentId, props) {
    console.log('DocumentStore._updateDocument ' + documentId + ' ' + JSON.stringify(props));
    var doc = this._documents[documentId];
    extend(doc, props);
  };

  this._documentExists = function(documentId) {
    console.log('DocumentStore._documentExists ' + documentId);
    return Boolean(this._documents[documentId]);
  };
};


DocumentStoreBase.extend(DocumentStore);

module.exports = DocumentStore;
