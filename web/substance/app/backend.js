var oo = require('substance/util/oo');
var $ = require('substance/util/jquery');
var _ = require('substance/util/helpers');
var Stencil = require('../model/Stencil');
var StencilNode = require('../model/StencilNode');

var CONFIG = {
  host: 'http://localhost:7373'
};

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

  var DOC_URL = CONFIG.host+"/core/stencils/examples/kitchensink";
  // var DOC_URL = 'data/kitchen-sink/index.html';

  // http://10.0.0.12:7373/core/stencils/examples/kitchensink/@content?format=
  this.getDocument = function(documentId, cb) {
    // TODO: we need a concept for generating the document URL
    var docUrl = DOC_URL;
    this._request('GET',  docUrl + "@content", null, function(err, resultStr) {
      if (err) { console.error(err); cb(err); }
      var result = JSON.parse(resultStr);
      var doc = new Stencil();
      doc.loadHtml(result.content);
      doc.id = documentId;
      doc.url = docUrl;
      window.doc = doc;
      cb(null, doc);
    });
  };

  // http://10.0.0.12:7373/core/stencils/examples/kitchensink/@save
  // http://10.0.0.12:7373/core/stencils/examples/kitchensink/@render
  this.saveDocument = function(doc, cb) {
    // Save the document, get the rendered html and update
    // generated properties
    var docUrl = DOC_URL;

    // EXPERIMENTAL: using @render endpoint to play around with generated properties
    this._request('PUT', docUrl + "@render", {
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
      _.each(tmp.getNodes(), function(copy, nodeId) {
        if (copy.constructor.static.generatedProps) {
          var node = doc.get(nodeId);
          if (!node) {
            console.warn('Node not present in document', nodeId);
            return;
          }
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