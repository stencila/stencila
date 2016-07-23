"use strict";

var DocumentNode = require('substance-fe0ed/model/DocumentNode');

function Cell() {
  Cell.super.apply(this, arguments);
}

Cell.Prototype = function() {

  this.isEmpty = function() {
    return !this.content;
  };

  this._updateDerivedProperties = function() {
    var content = this._content;
    var match = /^\s*([a-zA-Z0-9_@]+)?\s*(\=|\:|\^|\||\?|\~|\_)/.exec(content);
    delete this._expr;
    delete this._name;
    if (match) {
      if (match[1]) {
        this._name = match[1];
      }

      var symbol = match[2];
      this.kind = Cell.static.symbolToKind(symbol);

      this._expression = content.slice(match[0].length);
    } else {
      // In C++ we distinguish between different types of literals e.g. 'num, 'str'
      // but for now, just use 'lit' in JS for "constant" lieteral expression cells.
      this.kind = 'lit';
      this.value = content;
    }
  };

  this.getName = function() {
    return this._name;
  };

  /**
   * Get the source prefix for this cell
   *
   * The prefix is the name and symbol part of the cell
   * source. e.g. for `answer = 42`, `answer =` is the prefix.
   * Used for providing additional info on cells in UI.
   */
  this.getPrefix = function() {
    var name = this.getName() || '';
    var kind = this.kind;
    var symbol = Cell.static.kindToSymbol(kind);
    if (symbol) {
      if (name) return name + ' ' + symbol;
      else return symbol;
    }
    else if (name) return name;
    else return '';
  }

  this.isConstant = function() {
    return [
      'exp','map','req','man','tes','vis','cil'
    ].indexOf(this.kind)<0;
  };

  /**
   * Get the class name related to the display mode
   *
   * Defaults to `sm-clipped`
   */
  this.getDisplayClass = function() {
    if (this.displayMode=='exp') return 'sm-expanded';
    if (this.displayMode=='ove') return 'sm-overlay';
    if (this.displayMode=='cli') return 'sm-clipped';
    return 'sm-clipped';
  };

  // row and col indexes are managed by Table

  this.getRow = function() {
    return this.row;
  };

  this.getCol = function() {
    return this.col;
  };

  this.getCellId = function() {
    var Sheet = require('./Sheet');
    return Sheet.static.getCellId(this.row, this.col);
  };

};

DocumentNode.extend(Cell);

Cell.static.name = "sheet-cell";

Cell.static.defineSchema({
  // Kind of cell e.g. expression, maping, requirement, test
  kind: { type: "string", default: "" },

  // plain text (aka source)
  content: { type: "string", default: "" },

  // cell display mode
  displayMode: {type: "string", optional: true},

  // volatile data derived from table
  // ATM we need it as we set it during import
  // TODO: we should try to remove that from the schema
  row: "number",
  col: "number",

  // value is derived from the plain content by evaluating
  // it in an interpreter
  value: { type: "string", optional: true }, // evaluated value
  valueType: { type: "string", optional: true },
});

Cell.static.generatedProps = ['value', 'valueType'];

// whenever 'content' is changed we derive properties '_expr', '_name', etc.
Object.defineProperty(Cell.prototype, 'content', {
  get: function() {
    return this._content;
  },
  set: function(content) {
    this._content = content;
    this._updateDerivedProperties();
  }
});

Cell.static.kindToSymbol = function(kind) {
  if (kind=='exp') return '=';
  else if (kind=='map') return ':';
  else if (kind=='req') return '^';
  else if (kind=='man') return '|';
  else if (kind=='tes') return '?';
  else if (kind=='vis') return '~';
  else if (kind=='cil') return '_';
  else return '';
}

Cell.static.symbolToKind = function(symbol) {
  if (symbol=='=') return 'exp';
  else if (symbol==':') return 'map';
  else if (symbol=='^') return 'req';
  else if (symbol=='|') return 'man';
  else if (symbol=='?') return 'tes';
  else if (symbol=='~') return 'vis';
  else if (symbol=='_') return 'cil';
  else return '';
}

module.exports = Cell;
