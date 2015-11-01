var oo = require('substance/util/oo');
var $ = require('substance/util/jquery');
var _ = require('substance/util/helpers');
var Stencil = require('../model/Stencil');
var StencilNode = require('../model/StencilNode');

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
      var doc = new Stencil();
      doc.loadHtml(result.content);
      doc.id = documentId;
      window.doc = doc;
      cb(null, doc);
    });
  };

  // http://10.0.0.12:7373/core/stencils/examples/kitchensink/@save
  // http://10.0.0.12:7373/core/stencils/examples/kitchensink/@render
  this.saveDocument = function(doc, cb) {
    // Save the document, get the rendered html and update
    // generated properties
    var docAddress = DOC_ADDRESS;

    // EXPERIMENTAL: using @render endpoint to play around with generated properties
    this._request('PUT', this.apiUrl + docAddress + "@render", {
      'format': 'html',
      'content': doc.toHtml()
    }, function(err, resultStr) {
      if (err) return cb(err);
      var result = JSON.parse(resultStr);
      // creating a new document instance from the returned html
      // In future the server could provide a different format
      // containing only the rendered content as json
      var tmp = new Stencil();
      tmp.loadHtml(result.content);
      _.each(tmp.get('body').nodes, function(nodeId) {
        var node = doc.get(nodeId);
        var copy = tmp.get(nodeId);
        if (!node) {
          console.warn('Node not present in document', nodeId);
          return;
        }
        if (node instanceof StencilNode) {
          node.updateGeneratedProperties(copy);
        }
      });
    });
    console.warn('Not saving yet.');
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