'use strict';

var isNumber = require('lodash/lang/isNumber');
var each = require('lodash/collection/each');
var Document = require('substance/model/Document');
var DocumentSchema = require('substance/model/DocumentSchema');

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

  this._getDimension = function(nodes) {
    var nrows = 0;
    var ncols = 0;
    each(nodes, function(node) {
      if (node.type === "sheet-cell" && !node.isEmpty()) {
        nrows = Math.max(nrows, node.getRow());
        ncols = Math.max(ncols, node.getCol());
      }
    });
    return { rows: nrows+1, cols: ncols+1 };
  };

  this.getTableData = function(mode) {
    var nodes = this.getNodes();
    if (mode === "sparse") {
      nodes = nodes.filter(function(node) {
        return node.type === "sheet-cell" && !node.isEmpty();
      });
    }
    var tableData = this._getDimension(nodes);
    var cells = {};
    each(nodes, function(node) {
      if (node.type === "sheet-cell" && !node.isEmpty()) {
        cells[[node.getRow(), node.getCol()]] = node;
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
  if (!isNumber(col)) {
    throw new Error('Illegal argument.');
  }
  var name = "";
  while(true) {
    var mod = col % ALPHABET.length;
    col = Math.floor(col/ALPHABET.length);
    name = ALPHABET[mod] + name;
    if (col > 0) col--;
    else if (col === 0) break;
  }
  return name;
};

Sheet.static.getColumnIndex = function(col) {
  var index = 0;
  var rank = 1;
  each(col, function(letter) {
      index += rank * ALPHABET.indexOf(letter);
      rank++;
  });
  return index;
};

Sheet.static.getCellId = function(row,col) {
  return Sheet.static.getColumnName(col)+(row+1);
};

Sheet.static.getRowCol = function(id) {
  var match = /^([A-Z]+)([1-9][0-9]*)$/.exec(id);
  return [
    parseInt(match[2])-1,
    Sheet.static.getColumnIndex(match[1])
  ];
};

var _primitives = {
  'string': true,
  'int': true,
  'real': true
};

Sheet.isPrimitiveType = function(type) {
  return !!_primitives[type];
};

module.exports = Sheet;
