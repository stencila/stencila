'use strict';

var $ = require('substance/util/jquery');
var OO = require('substance/util/oo');
var Document = require('substance/model/Document');
var DocumentSchema = require('substance/model/DocumentSchema');
var HtmlImporter = require('substance/model/HtmlImporter');
var HtmlExporter = require('substance/model/HtmlExporter');

var StencilDefaultNode = require('../packages/default/StencilDefaultNode');

// Default Schema
// ----------------

var defaultSchema = new DocumentSchema("stencil", "0.1.0");

defaultSchema.getDefaultTextType = function() {
  return "paragraph";
};

defaultSchema.addNodes([
  // General nodes
  require('substance/packages/paragraph/Paragraph'),
  require('substance/packages/heading/Heading'),
  require('substance/packages/emphasis/Emphasis'),
  require('substance/packages/strong/Strong'),
  require('substance/packages/link/Link'),

  // Stencil-specific nodes
  require('../packages/title/StencilTitle'),
  require('../packages/summary/StencilSummary'),

  require('../packages/math/StencilMath'),
  require('../packages/equation/StencilEquation'),

  require('../packages/exec/StencilExec'),
  require('../packages/figure/StencilFigure'),
  require('../packages/text/StencilText'),

  StencilDefaultNode
]);


// Importer
// ----------------

function Importer(schema) {
  Importer.super.call(this, { schema: schema });
}

Importer.Prototype = function() {
  this.convert = function($rootEl, doc) {
    this.initialize(doc, $rootEl);
    this.convertContainer($rootEl, 'body');
    this.finish();
  };

  this.defaultConverter = function($el, converter) {
    /* jshint unused:false */
    var node = StencilDefaultNode.static.fromHtml($el, converter);
    node.type = 'stencil-default-node';
    return node;
  };
};

OO.inherit(Importer, HtmlImporter);

// Exporter
// ----------------

function Exporter(schema) {
  Exporter.super.call(this, { schema: schema });
}

Exporter.Prototype = function() {

  this.convert = function(doc, options) {
    this.initialize(doc, options);

    var body = doc.get('body');
    var bodyNodes = this.convertContainer(body);
    var $el = $('<div>');
    $el.append(bodyNodes);
    return $el.html();
  };
};

OO.inherit(Exporter, HtmlExporter);


// Stencil
// ----------------

var Stencil = function(schema) {
  Document.call(this, schema || defaultSchema);
};

Stencil.Prototype = function() {

  this.initialize = function() {
    Document.prototype.initialize.apply(this, arguments);

    this.create({
      type: "container",
      id: "body",
      nodes: []
    });
  };

  this.toHtml = function() {
    return new Exporter(this.schema).convert(this);
  };

  // replaces the content by loading from the given html
  this.loadHtml = function(html) {
    // Deletes all nodes (including container node 'body')
    this.clear();

    // Disable transaction enforcement until the import is finished
    this.FORCE_TRANSACTIONS = false;

    // Recreate body container
    // TODO: find a better solution
    this.create({
      type: "container",
      id: "body",
      nodes: []
    });

    var $content = $('<div>').html(html);
    new Importer(this.schema).convert($content, this);
    // sets this.FORCE_TRANSACTIONS = true again
    this.documentDidLoad();
  };

  this.getTOCNodes = function() {
    var tocNodes = [];
    var contentNodes = this.get('body').nodes;
    contentNodes.forEach(function(nodeId) {
      var node = this.get(nodeId);
      if (node.type === "heading") {
        tocNodes.push(node);
      }
    }.bind(this));
    return tocNodes;
  };

  this.getCila = function(cb) {
    window.__backend.cilaGet(this,function(error, result){
      console.log(result);
      cb(result);
    });
  };
};

OO.inherit(Stencil, Document);
Stencil.schema = defaultSchema;

Stencil.Importer = Importer;
module.exports = Stencil;
