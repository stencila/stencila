'use strict';

var SnapshotEngineBase = require('substance/collab/SnapshotEngine');

/**
 * Handles computation of snapshots for Stencila component sessions.
 * 
 * This extends `substance/collab/SnapshotEngine` to handle the creation of
 * alternative document types (at time of writing this base class seemed
 * to be able to only create one type of document based on a configurator)
 *
 * @class      SnapshotEngine (name)
 * @param      {<type>}  config  The configuration
 */
function SnapshotEngine(config) {
  SnapshotEngine.super.apply(this, arguments);

  this.documentFactory = config.documentFactory;
}

SnapshotEngine.Prototype = function() {

  this._createDocumentInstance = function(schemaName) {
    return this.documentFactory.create(schemaName);
  };

};

SnapshotEngineBase.extend(SnapshotEngine);

module.exports = SnapshotEngine;
