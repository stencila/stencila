var Stencil = require('../model/Stencil');
var oo = require('substance/util/oo');
var $ = require('substance/util/jquery');
var _ = require('substance/util/helpers');

var Backend = function() {
  this.apiUrl = "http://10.0.0.12:7373";
};

Backend.Prototype = function() {

  // A generic request method
  // -------------------
  //
  // Deals with sending the authentication header, encoding etc.

  this._request = function(method, url, data, cb) {
    var ajaxOpts = {
      type: method,
      url: url,
      contentType: "application/json; charset=UTF-8",
      // dataType: "json",
      success: function(data) {
        cb(null, data);
      },
      error: function(err) {
        console.error(err);
        cb(err.responseText);
      }
    };

    if (data) {
      ajaxOpts.data = JSON.stringify(data);
    }

    $.ajax(ajaxOpts);
  };

  // Document
  // ------------------
  var DOC_ADDRESS = "//home/nokome/stencila/source/stencila/web/substance/app/data/kitchen-sink";

  // http://10.0.0.12:7373/core/stencils/examples/kitchensink/@content?format=
  this.getDocument = function(documentId, cb) {
    var docAddress = DOC_ADDRESS;
    this._request('GET', this.apiUrl + docAddress + "@content", null, function(err, resultStr) {
      if (err) { console.error(err); cb(err); }
      var result = JSON.parse(resultStr);
      debugger;
      var doc = new Stencil();
      doc.loadHtml(result.content);
      doc.id = documentId;
      window.doc = doc;
      cb(null, doc);
    });
  };

  this.saveDocument = function(doc, cb) {
    cb('Not supported in dev version');
  };

  // Figure related
  // ------------------

  this.uploadFigure = function(file, cb) {
    // This is a fake implementation
    var objectURL = window.URL.createObjectURL(file);
    cb(null, objectURL);
  };
};

oo.initClass(Backend);

module.exports = Backend;