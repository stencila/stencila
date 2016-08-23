"use strict";

var DocumentEngineBase = require('substance/collab/DocumentEngine');
var ObjectOperation = require('substance/model/data/ObjectOperation');
var JSONConverter = require('substance/model/JSONConverter');
var Err = require('substance/util/SubstanceError');

function DocumentEngine(config) {
  DocumentEngine.super.apply(this, arguments);

  this.modelFactory = config.modelFactory;
}

DocumentEngine.Prototype = function() {

  /**
   * Create a document
   * 
   * 
   */
  this.createDocument = function(args, cb) {
    // We start with version 1 because version 0 is assumed to be empty and
    // is treated specially by Substance `SnapshotEngine`
    var version = 1;

    // Create a record in the `DocumentStore`
    this.documentStore.createDocument({
      schemaName: args.schemaName,
      documentId: args.documentId,
      version: version
    }, function(err, documentRecord) {
      if(err) return cb(new Err('CreateError', { message: err }));

      // Create a "no op" change in the `ChangeStore` so that version 1 has 
      // one change associated with it
      this.changeStore.addChange({
        documentId: documentRecord.documentId,
        change: {
          ops: [
            {type: ObjectOperation.NOP}
          ]
        }
      }, function(err, version) {
        if(err) return cb(new Err('CreateError', { message: err }));

        // Save a snapshot of the document
        // Necessary because our version 1 of a document actually has stuff in it
        // and we want to use it as the "baseline" when snapshots are requested
        this.snapshotEngine.snapshotStore.saveSnapshot({
          documentId: documentRecord.documentId,
          version: documentRecord.version,
          data: args.data
        }, function(err, snapshot) {
          if(err) return cb(new Err('ShapshotError', { message: err }));

          cb(null, snapshot);
        });

      }.bind(this));

    }.bind(this));

  };

};

DocumentEngineBase.extend(DocumentEngine);

module.exports = DocumentEngine;
