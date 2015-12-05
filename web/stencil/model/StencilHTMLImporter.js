'use strict';

var HTMLImporter = require('substance/model/HTMLImporter');
var DefaultDOMElement = require('substance/ui/DefaultDOMElement');
var Stencil = require('./Stencil');
var converters = require('./StencilHTMLConverters');
var StencilDefaultNodeHTMLConverter = require('../packages/default/StencilDefaultNodeHTMLConverter');

function StencilHTMLImporter() {
  StencilHTMLImporter.super.call(this, {
    schema: Stencil.schema,
    converters: converters,
    DocumentClass: Stencil,
    containerId: 'body'
  });
}

StencilHTMLImporter.Prototype = function() {

  var _super = StencilHTMLImporter.super.prototype;

  this.importDocument = function(html) {
    // initialization
    this.reset();
    // Stencil is providing the content of body, not a full HTML document
    var elements = DefaultDOMElement.parseHTML(html);
    this.convertContainer(elements, 'body');
    var doc = this.generateDocument();
    return doc;
  };

  this._getConverterForElement = function() {
    var converter = _super._getConverterForElement.apply(this, arguments);
    converter = converter || StencilDefaultNodeHTMLConverter;
    return converter;
  };

};

HTMLImporter.extend(StencilHTMLImporter);

module.exports = StencilHTMLImporter;