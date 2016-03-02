"use strict";

var DocumentNode = require('substance/model/DocumentNode');

function Cell() {
  Cell.super.apply(this, arguments);
}

Cell.Prototype = function() {

  this.isEmpty = function() {
    return !this.content;
  };

  this._updateDerivedProperties = function() {
    var content = this._content;
    var match = /^\s*([a-zA-Z0-9_@]+)?\s*(\=|\~|\^|\:|\?|\|)/.exec(content);
    delete this._expr;
    delete this._name;
    if (match) {
      if (match[1]) {
        this._name = match[1];
      }

      var symbol = match[2];
      if (symbol=='=') this.kind = 'exp';
      else if (symbol=='~') this.kind = 'map';
      else if (symbol=='^') this.kind = 'req';
      else if (symbol==':') this.kind = 'man';
      else if (symbol=='?') this.kind = 'tes';
      else if (symbol=='|') this.kind = 'vis';

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
    var symbol = '';
    if(kind) {
      if (kind=='exp') symbol = '=';
      else if (kind=='map') symbol = '~';
      else if (kind=='req') symbol = '^';
      else if (kind=='man') symbol = ':';
      else if (kind=='tes') symbol = '?';
      else if (kind=='vis') symbol = '|';
      else symbol = '';
    }
    if (symbol) {
      if (name) return name + ' ' + symbol;
      else return symbol;
    }
    else if (name) return name;
    else return '';
  }

  this.isConstant = function() {
    return this.kind == 'lit';
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

module.exports = Cell;
