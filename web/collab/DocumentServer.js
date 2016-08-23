'use strict';

var DocumentServerBase = require('substance/collab/DocumentServer');

/**
 * A custom `DocumentServer` to match our custom `DocumentEngine`.
 * Handles necessary extra endpoint arguments and adds endpoints.
 *
 * @class      DocumentServer (name)
 * @param      {<type>}  config  The configuration
 */
function DocumentServer(config) {
  DocumentServer.super.apply(this, arguments);
}

DocumentServer.Prototype = function() {

  var _super = DocumentServer.super.prototype;

  this.bind = function(app) {
    // These bindings allow for path to be '/' (the bindings in the base class don't)
    app.post(this.path, this._createDocument.bind(this));
    app.get(this.path + ':id', this._getDocument.bind(this));
    app.get(this.path, this._listDocuments.bind(this));
    app.delete(this.path + ':id', this._deleteDocument.bind(this));
  };

  this._createDocument = function(req, res, next) {
    this.engine.createDocument({
      schemaName: req.body.schemaName,
      documentId: req.body.documentId,
      data: req.body.data
    }, function(err, result) {
      if (err) return next(err);
      res.json(result);
    });
  };

  this._listDocuments = function(req, res, next) {
    this.engine.listDocuments(function(err, result) {
      if (err) return next(err);
      res.json(result);
    });
  };

};

DocumentServerBase.extend(DocumentServer);

module.exports = DocumentServer;
