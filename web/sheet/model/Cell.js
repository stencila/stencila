"use strict";

var DocumentNode = require('substance/model/DocumentNode');

function Cell(){
  Cell.super.apply(this, arguments);
}

Cell.Prototype = function() {

  this.isEmpty = function() {
    return !this.source;
  };

  this.getName = function() {
    var match = /^\s*([a-zA-Z0-9_@])=/.exec(this.source);
    if (match) {
      return match[1];
    }
  };

  this.isPrimitive = function() {
    switch(this.valueType) {
      case 'string':
      case 'real':
      case 'int':
        return true;
      default:
        return false;
    }
  };

  this.getValue = function() {
    if (this.isPrimitive()) {
      return this.source;
    } else {
      return this.value;
    }
  };

  this.getCellId = function() {
    var Sheet = require('./Sheet');
    return Sheet.static.getCellId(this.row, this.col);
  };

  // row and col indexes are managed by Table

  this.setRow = function(row) {
    this.row = row;
    return this;
  };

  this.getRow = function() {
    return this.row;
  };

  this.setCol = function(col) {
    this.col = col;
    return this;
  };

  this.getCol = function() {
    return this.col;
  };

};

DocumentNode.extend(Cell);

Cell.static.name = "sheet-cell";

Cell.static.defineSchema({
  source: "text", // the raw string content as you would see in TSV file

  displayMode: {type: "string", optional: true},

  // volatile data derived from source
  value: { type: "string", optional: true }, // evaluated value
  contentType: { type: "string", default: 'text' }
});

Cell.static.generatedProps = ['value'];

module.exports = Cell;
