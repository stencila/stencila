'use strict';

var ChangeStoreBase = require('substance/collab/ChangeStore');

/**
 * Stores changes to Stencila component jams.
 * 
 * Used to sync component jam sessions across collaborators.
 *
 * @class      ChangeStore (name)
 */
function ChangeStore() {
  ChangeStore.super.apply(this, arguments);

  this._changes = [];
}

ChangeStore.Prototype = function() {

  this._deleteChanges = function(documentId) {
    console.log('ChangeStore._deleteChanges ' + documentId);
    var changes = this._getChanges(documentId);
    delete this._changes[documentId];
    return changes;
  };

  this._getVersion = function(documentId) {
    console.log('ChangeStore._getVersion ' + documentId);
    var changes = this._changes[documentId];
    return changes ? changes.length : 0;
  };

  this._getChanges = function(documentId) {
    console.log('ChangeStore._getChanges ' + documentId);
    return this._changes[documentId] || [];
  };

  this._addChange = function(documentId, change) {
    console.log('ChangeStore._addChange ' + documentId);
    if (!this._changes[documentId]) {
      this._changes[documentId] = [];
    }
    this._changes[documentId].push(change);
  };
};

ChangeStoreBase.extend(ChangeStore);

module.exports = ChangeStore;
