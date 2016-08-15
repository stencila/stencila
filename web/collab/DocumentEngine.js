"use strict";

var DocumentEngineBase = require('substance/collab/DocumentEngine');
var JSONConverter = require('substance/model/JSONConverter');
var Err = require('substance/util/SubstanceError');

function DocumentEngine(config) {
  DocumentEngine.super.apply(this, arguments);

  this.modelFactory = config.modelFactory;
}

DocumentEngine.Prototype = function() {

  this.getDocument = function(args, cb) {

    this.documentStore.documentExists(args.documentId, function(err, exists) {
      if (exists) {
        // If the jam exists then just get it from the snapshot engine
        this.snapshotEngine.getSnapshot(args, cb);
      } else {
        // Otherwise, we need to create a jam by importing the document...
        
        var address = 'tests/document/nodes/paragraph';

        // Resolve the Stencila address into a Stencila component type
        // TDO
        var schemaName = 'stencila-document';

        // Read the document from repository
        this.modelFactory.readDocument(address, function(err, doc) {

          if(err) {
            return cb(new Err('ReadError', {
              cause: err
            }));
          }

          var json = this.modelFactory.exportJson(doc);

          // Create the jam in the `DocumentStore`
          this.documentStore.createDocument({
            schemaName: schemaName,
            documentId: args.documentId,
            version: 0
          }, function(err, documentRecord) {

            if (err) {
              return cb(new Err('CreateError', {
                cause: err
              }));
            }

            cb(null, {
              documentId: documentRecord.documentId,
              version: documentRecord.version,
              data: json,
            });

          }.bind(this));

        }.bind(this));

      }

    }.bind(this));

  };

};

DocumentEngineBase.extend(DocumentEngine);

module.exports = DocumentEngine;
