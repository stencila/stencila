'use strict';

var HTMLImporter = require('substance/model/HTMLImporter');
var Stencil = require('./Stencil');
var converters = require('./StencilHTMLConverters');

function StencilImporter() {
  StencilImporter.super.call(this, {
    schema: Stencil.schema,
    converters: converters,
    DocumentClass: Stencil,
    containerId: 'body'
  });
}

StencilImporter.Prototype = function() {
  this.convertDocument = function(documentEl) {
    // Import main container
    var bodyNodes = documentEl.find('main').children;
    this.convertContainer(bodyNodes, 'body');
  };
};

HTMLImporter.extend(StencilImporter);

module.exports = StencilImporter;