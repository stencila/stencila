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

  this.connect(this, {
    'document:changed': this._onChange
  });
};

Sheet.Prototype = function() {

  this.getCellAt = function(rowIdx, colIdx) {
    if (!this._matrix) this._computeMatrix();

    var row = this._matrix[rowIdx];
    if (row) {
      var cellId = row[colIdx];
      if (cellId) {
        return this.get(cellId);
      }
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
    this._matrix = {};
    this._nrows = 0;
    this._ncols = 0;
    each(this.getNodes(), function(node) {
      if (node.type === "sheet-cell") {
        this._updateCellMatrix(node.row, node.col, node.id);
      }
    }.bind(this));
  };

  this._updateCellMatrix = function(rowIdx, colIdx, cellId) {
    var row = this._matrix[rowIdx];
    if (!row) {
      row = {};
      this._matrix[rowIdx] = row;
    }
    if (cellId) {
      row[colIdx] = cellId;
    } else {
      delete row[colIdx];
    }
    this._nrows = Math.max(this._nrows, rowIdx+1);
    this._ncols = Math.max(this._ncols, colIdx+1);
  };

  // updating the matrix whenever a cell has been created or deleted
  this._onChange = function(change) {
    if (!this._matrix) return;
    each(change.created, function(cell) {
      this._updateCellMatrix(cell.row, cell.col, cell.id);
    }.bind(this));
    each(change.deleted, function(cell) {
      this._updateCellMatrix(cell.row, cell.col, null);
    }.bind(this));
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

module.exports = Sheet;
