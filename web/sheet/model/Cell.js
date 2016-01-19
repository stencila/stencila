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

};

DocumentNode.extend(Cell);

Cell.static.name = "sheet-cell";

Cell.static.defineSchema({
  row: "number",
  col: "number",
  source: "text", // the raw string content as you would see in TSV file

  cid: "string", // such as A1?

  // volatile data (derived from source)
  value: { type: "string", optional: true }, // evaluated value
  valueType: "string",
});

Cell.static.generatedProps = ['value', 'valueType'];

module.exports = Cell;
