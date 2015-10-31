var Stencil = require('../model/Stencil');
var oo = require('substance/util/oo');
var $ = require('substance/util/jquery');

var Backend = function() {

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

  this.getDocument = function(documentId, cb) {
    this._request('GET', 'data/kitchen-sink/index.html', null, function(err, rawDoc) {
      if (err) { console.error(err); cb(err); }
      var doc = new Stencil();
      doc.loadHtml(rawDoc);
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