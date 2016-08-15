'use strict';

var DocumentStoreBase = require('substance/collab/DocumentStore');
var extend = require('lodash/object/extend');

/**
 * Stores Stencila component jams.
 * 
 * Used to sync component jam sessions across collaborators.
 *
 * @class      DocumentStore (name)
 * @param      {<type>}  config  The configuration
 */
function DocumentStore(config) {
  DocumentStore.super.apply(this, arguments);

  this._jams = {
  };
}

DocumentStore.Prototype = function() {

  this._createDocument = function(props) {
    console.log('DocumentStore._createDocument' + JSON.stringify(props));
    this._jams[props.documentId] = props;
  };

  this._deleteDocument = function(documentId) {
    console.log('DocumentStore._deleteDocument ' + documentId);
    delete this._jams[documentId];
  };

  this._getDocument = function(documentId) {
    console.log('DocumentStore._getDocument ' + documentId);
    return this._jams[documentId];
  };

  this._updateDocument = function(documentId, props) {
    var doc = this._jams[documentId];
    extend(doc, props);
  };

  this._documentExists = function(documentId) {
    return Boolean(this._jams[documentId]);
  };
};


DocumentStoreBase.extend(DocumentStore);

module.exports = DocumentStore;
