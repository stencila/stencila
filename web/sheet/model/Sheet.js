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

  this._matrix = null;
  this._nrows = 0;
  this._ncols = 0;
};

Sheet.Prototype = function() {

  this.getCellAt = function(rowIdx, colIdx) {
    if (!this._matrix) this._computeMatrix();

    var row = this._matrix[rowIdx]
    if (row) {
      return row[colIdx];
    }
    return null;
  };

  this.getRowCount = function() {
    if (!this._matrix) this._computeMatrix();

    return this._nrows;
  };

  this.getColumnCount = function() {
    if (!this._matrix) this._computeMatrix();

    return this._ncols;
  };

  this._computeMatrix = function() {
    var matrix = [];
    var nrows = 0;
    var ncols = 0;
    each(this.getNodes(), function(node) {
      if (node.type === "sheet-cell") {
        var cell = node;
        var rowIdx = cell.getRow();
        var colIdx = cell.getCol();
        var row = matrix[rowIdx];
        if (!row) {
          row = [];
          matrix[rowIdx] = row;
        }
        row[colIdx] = cell;
        nrows = Math.max(nrows, rowIdx);
        ncols = Math.max(ncols, colIdx);
      }
    });
    this._matrix = matrix;
    this._nrows = nrows + 1;
    this._ncols = ncols + 1;
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

Sheet.normalizeValueType = function(type) {
  if (type === 'ImageFile') {
    return 'image';
  }
  return type;
};

module.exports = Sheet;
