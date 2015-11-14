var oo = require('substance/util/oo');
var $ = require('substance/util/jquery');
var _ = require('substance/util/helpers');
var Stencil = require('../model/Stencil');

var CONFIG = {
  host: 'http://localhost:7373'
};

var Backend = function() {
  this._getHost();
  this._getAddress();
};

Backend.Prototype = function() {

  // A generic request method
  // -------------------
  //
  // Deals with sending the authentication header, encoding etc.

  this._request = function(method, endpoint, data, cb) {
    var ajaxOpts = {
      type: method,
      url: this.protocol+'//'+this.host+':'+this.port+'/'+this.address+'@'+endpoint,
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

  this._getHost = function() {
    var location = window.location;
    this.protocol = location.protocol;
    if(this.protocol==='file:') this.host = 'localfile';
    else this.host = location.hostname;
    this.port = location.port;
  };

  this._getAddress = function() {
    // Address
    this.address = null;
    // ... from <meta> tag
    var address = $('head meta[itemprop=address]');
    if(address.length) this.address = address.attr('content');
    // ... or from url
    if(!this.address) {
      // Remove the leading /
      var path = window.location.pathname.substr(1);
      // Remove the last part of path if it is a title slug
      var lastIndex = path.lastIndexOf('/');
      var last = path.substr(lastIndex);
      if(last.substr(last.length-1)=="-") this.address = path.substr(0,lastIndex);
    }
  };


  // Document
  // ------------------

  // http://10.0.0.12:7373/core/stencils/examples/kitchensink/@content?format=
  this.getDocument = function(documentId, cb) {
    var address = this.address;
    // TODO: we need a concept for generating the document URL
    this._request('GET', "content", null, function(err, resultStr) {
      if (err) { console.error(err); cb(err); }
      var result = JSON.parse(resultStr);
      var doc = new Stencil();
      doc.loadHtml(result.content);
      doc.id = documentId;
      doc.url = address;
      window.doc = doc;
      cb(null, doc);
    });
  };

  this.saveDocument = function(doc, cb) {
    this._request('PUT', "save", {
      'format': 'html',
      'content': doc.toHtml()
    }, function(err, resultStr) {
      if (err) { console.error(err); cb(err); }
      cb(null);
    });
  };

  this.renderDocument = function(doc, cb) {
    this._request('PUT', "render", {
      'format': 'html',
      'content': doc.toHtml()
    }, function(err, resultStr) {
      if (err) { console.error(err); cb(err); }
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
      cb(null);
    });
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
