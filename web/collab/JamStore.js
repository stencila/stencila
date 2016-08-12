'use strict';

var DocumentStore = require('substance/collab/DocumentStore');
var extend = require('lodash/object/extend');

/**
 * Stores Stencila component jams.
 * 
 * Used to sync component jam sessions across collaborators.
 *
 * @class      JamStore (name)
 * @param      {<type>}  config  The configuration
 */
function JamStore(config) {
  JamStore.super.apply(this, arguments);

  this._jams = {
    
    'default:comment' : {
      id: 'default:comment',
      schemaName: 'stencila-document',
      version: 0
    },

    'default:edit' : {
      id: 'default:edit',
      schemaName: 'stencila-document',
      version: 0
    }
    
  };
}

JamStore.Prototype = function() {

  this._createDocument = function(props) {
    console.log('JamStore._createDocument' + JSON.stringify(props));
    this._jams[props.documentId] = props;
  };

  this._deleteDocument = function(documentId) {
    console.log('JamStore._deleteDocument ' + documentId);
    delete this._jams[documentId];
  };

  this._getDocument = function(documentId) {
    console.log('JamStore._getDocument ' + documentId);
    return this._jams[documentId];
  };

  this._updateDocument = function(documentId, props) {
    console.log('JamStore._updateDocument ' + documentId + ' ' + JSON.stringify(props));
    var doc = this._jams[documentId];
    extend(doc, props);
  };

  this._documentExists = function(documentId) {
    console.log('JamStore._documentExists ' + documentId);
    return Boolean(this._jams[documentId]);
  };
};


DocumentStore.extend(JamStore);

module.exports = JamStore;
