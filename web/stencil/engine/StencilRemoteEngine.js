'use strict';

var each = require('lodash/collection/each');

var RemoteEngine = require('../../shared/RemoteEngine');

var StencilHTMLImporter = require('../model/StencilHTMLImporter');
var StencilHTMLExporter = require('../model/StencilHTMLExporter');
var importer = new StencilHTMLImporter();
var exporter = new StencilHTMLExporter();

function StencilRemoteEngine() {
  StencilRemoteEngine.super.apply(this, arguments);
  window._engine = this;
}

StencilRemoteEngine.Prototype = function() {

  this.save = function(doc, callback) {
    this._request('PUT', 'content', {
      'format': 'html',
      'content': exporter.exportDocument(doc)
    }, callback);
  };

  this.render = function(doc, callback) {
    this._request('PUT', 'render', {
      'format': 'html',
      'content': exporter.exportDocument(doc)
    }, function(err, result) {
      if (err) callback(err);
      // Create a new document instance from the returned html
      // In future the server could provide a different format
      // containing only the rendered content as json
      var tmp = importer.importDocument(result.content);
      each(tmp.getNodes(), function(copy, nodeId) {
        if (copy.constructor.static.generatedProps) {
          var node = doc.get(nodeId);
          if (!node) {
            console.warn('Node not present in document', nodeId);
            return;
          }
          node.updateGeneratedProperties(copy);
        }
      });
      callback(null);
    });
  };

  this.cilaGet = function(doc, cb){
    // FIXME For compatability with C+++ backend
    // this temporarily needs to be a POST but will
    // eventually be a GET
    this._request('PUT', 'content', {
      'format': 'cila'
    }, function(err, result) {
      if (err) { console.error(err); cb(err); }
      cb(null,result.content);
    });
  };

};

RemoteEngine.extend(StencilRemoteEngine);

module.exports = StencilRemoteEngine;
