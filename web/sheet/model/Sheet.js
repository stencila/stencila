'use strict';

var Document = require('substance/model/Document');
var DocumentSchema = require('substance/model/DocumentSchema');
var _ = require('substance/util/helpers');

var defaultSchema = new DocumentSchema("stencila-sheet", "0.1.0");

// TODO: defaultTextType does not make sense in this application.
// Still we need to specify it here as otherwise there is an exception
// because this method is abstract, but gets called by DOMImporter constructor.
defaultSchema.getDefaultTextType = function() {
  return "paragraph";
};

defaultSchema.addNodes([
  require('./Cell'),
//   // General nodes
//   // require('substance/packages/emphasis/Emphasis'),
//   // require('substance/packages/strong/Strong'),
//   // require('substance/packages/link/Link'),
//   // require('../../stencil/packages/math/StencilMath'),
//   // require('../../stencil/packages/text/StencilText')
]);

var Sheet = function(schema) {
  Document.call(this, schema || defaultSchema);
};

Sheet.Prototype = function() {

  this.getCila = function(cb) {
    window.__backend.cilaGet(this,function(error, result){
      cb(result);
    });
  };

  this._getDimension = function(nodes) {
    var nrows = 0;
    var ncols = 0;
    _.each(nodes, function(node) {
      if (node.content && node.type === "sheet-cell") {
        nrows = Math.max(nrows, node.row);
        ncols = Math.max(ncols, node.col);
      }
    });
    return { rows: nrows+1, cols: ncols+1 };
  };

  this.getTableData = function(mode) {
    var nodes = this.getNodes();
    if (mode === "sparse") {
      nodes = nodes.filter(function(node) {
        return node.type === "sheet-cell" && node.content;
      });
    }
    var tableData = this._getDimension(nodes);
    var cells = {};
    _.each(nodes, function(node) {
      if (node.content && node.type === "sheet-cell") {
        cells[[node.row, node.col]] = node;
      }
    });
    tableData.cells = cells;
    return tableData;
  };

};

Document.extend(Sheet);

Sheet.static.schema = defaultSchema;

var ALPHABET = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

Sheet.static.getColumnName = function(col) {
  var name = "";
  do {
    var mod = col % ALPHABET.length;
    name += ALPHABET[mod];
    col -= ALPHABET.length;
  } while (col > 0);
  return name;
};


module.exports = Sheet;
