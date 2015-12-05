
'use strict';

var HTMLExporter = require('substance/model/HTMLExporter');
var converters = require('./StencilHTMLConverters');

function StencilHTMLExporter() {
  StencilHTMLExporter.super.call(this, {
    converters: converters,
    containerId: 'body'
  });
}

StencilHTMLExporter.Prototype = function() {
  this.exportDocument = function(doc) {
    var bodyNodes = this.convertContainer(doc.get('body'));
    var wrapper = this.$$("div").append(bodyNodes);
    return wrapper.html();
  };
};

HTMLExporter.extend(StencilHTMLExporter);

module.exports = StencilHTMLExporter;