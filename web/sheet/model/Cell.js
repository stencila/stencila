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
    var match = /^\s*([a-zA-Z0-9_@]+)?=/.exec(content);
    delete this._expr;
    delete this._name;
    if (match) {
      if (match[1]) {
        this._contentType = 'named-expression';
        this._name = match[1];
      } else {
        this._contentType = 'expression';
      }
      this._expression = content.slice(match[0].length);
    } else {
      this._contentType = 'primitive';
      this.value = content;
    }
  };

  this.getName = function() {
    return this._name;
  };

  this.isPrimitive = function() {
    console.warn('isPrimitive is deprecated. Use isConstant');
    return this.isConstant();
  };

  this.isConstant = function() {
    return this._contentType === "primitive";
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
  // plain text (aka source)
  content: { type: "string", default: "" },

  // cell display mode
  displayMode: {type: "string", default: 'clipped'},

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
