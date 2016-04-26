'use strict';

var oo = require('substance/util/oo');
var Document = require('substance/model/Document');
var DocumentSchema = require('substance/model/DocumentSchema');

var StencilDefaultNode = require('../packages/default/StencilDefaultNode');

// Default Schema
// ----------------

var defaultSchema = new DocumentSchema("stencil", "0.1.0");

defaultSchema.getDefaultTextType = function() {
  return "paragraph";
};

defaultSchema.addNodes([
  // Substance nodes, in alphabetical order, from `substance/packages`
  require('substance/packages/blockquote/Blockquote'),
  require('substance/packages/code/Code'),
  //require('substance/packages/codeblock/Codeblock'),
  require('substance/packages/emphasis/Emphasis'),
  require('substance/packages/heading/Heading'),
  require('substance/packages/image/Image'),
  require('substance/packages/link/Link'),
  require('substance/packages/list/List'),
  require('substance/packages/paragraph/Paragraph'),
  require('substance/packages/strong/Strong'),
  require('substance/packages/subscript/Subscript'),
  require('substance/packages/superscript/Superscript'),
  require('substance/packages/table/Table'),
  require('substance/packages/table/TableSection'),
  require('substance/packages/table/TableRow'),
  require('substance/packages/table/TableCell'),

  // Stencil-specific nodes
  require('../packages/title/StencilTitle'),
  require('../packages/summary/StencilSummary'),

  require('../packages/math/StencilMath'),
  require('../packages/equation/StencilEquation'),
  require('../packages/codeblock/StencilCodeblock'),

  require('../packages/exec/StencilExec'),
  require('../packages/parameter/StencilParameter'),
  require('../packages/figure/StencilFigure'),
  require('../packages/include/StencilInclude'),
  require('../packages/text/StencilText'),

  StencilDefaultNode
]);


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
    window._engine.cilaGet(this,function(error, result){
      cb(result);
    });
  };
};

oo.inherit(Stencil, Document);
Stencil.schema = defaultSchema;

module.exports = Stencil;
