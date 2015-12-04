var oo = require('substance/util/oo');
var $ = require('substance/util/jquery');
var _ = require('substance/util/helpers');
var Stencil = require('./model/Stencil');

var StencilHTMLImporter = require('./model/StencilHTMLImporter');
var StencilHTMLExporter = require('./model/StencilHTMLExporter');
var importer = new StencilHTMLImporter();
var exporter = new StencilHTMLExporter();

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
      // Specify JSON as content type to send
      contentType: "application/json; charset=UTF-8",
      // Type of data expected back
      // "json": Evaluates the response as JSON and returns a JavaScript object.
      dataType: "json",
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

  this.getDocument = function(documentId, cb) {
    var address = this.address;
    // TODO: we need a concept for generating the document URL
    this._request('GET', "content", null, function(err, result) {
      if (err) { console.error(err); cb(err); }

      var doc = importer.importDocument(result.content);
      doc.id = documentId;
      doc.url = address;
      window.doc = doc;
      cb(null, doc);
    });
  };

  this.saveDocument = function(doc, cb) {
    this._request('PUT', 'content', {
      'format': 'html',
      'content': exporter.exportDocument(doc)
    }, function(err) {
      if (err) { console.error(err); cb(err); }
      cb(null);
    });
  };

  this.activate = function(doc, cb){
    this._request('PUT', 'activate', function(err) {
      if (err) { console.error(err); cb(err); }
      cb(null);
    });
  };

  this.renderDocument = function(doc, cb) {
    this._request('PUT', "render", {
      'format': 'html',
      'content': exporter.exportDocument(doc)
    }, function(err, result) {
      if (err) { console.error(err); cb(err); }
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

  this.cilaGet = function(doc, cb){
    // FIXME For compatability with C+++ backend
    // this temporarily needs to be a POST but will
    // eventually be a GET
    this._request('POST', 'content', {
      'format': 'cila'
    }, function(err, result) {
      if (err) { console.error(err); cb(err); }
      cb(null,result.content);
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
