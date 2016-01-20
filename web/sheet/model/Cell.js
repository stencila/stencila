"use strict";

var DocumentNode = require('substance/model/DocumentNode');
var includes = require('lodash/collection/includes');

function Cell() {
  Cell.super.apply(this, arguments);
}

Cell.Prototype = function() {

  this.isEmpty = function() {
    return !this.content;
  };

  this.getName = function() {
    var match = /^\s*([a-zA-Z0-9_@])=/.exec(this.content);
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
      return this.content;
    } else {
      return this.value;
    }
  };

  this.getCellId = function() {
    var Sheet = require('./Sheet');
    return Sheet.static.getCellId(this.row, this.col);
  };

  // row and col indexes are managed by Table

  this.getRow = function() {
    return this.row;
  };

  this.getCol = function() {
    return this.col;
  };

  /**
    Used to determine content rendering component (TextComponent, ImageComponent etc.)
    
    @return {String} 'text', 'image' or 'object'
  */
  this.getContentType = function() {
    if (includes(['integer', 'real', 'string'], this.valueType)) {
      return 'text';
    } else if (this.valueTypetype === 'ImageFile') {
      return 'image';
    } else {
      return 'object';
    }
  };

};

DocumentNode.extend(Cell);

Cell.static.name = "sheet-cell";

Cell.static.defineSchema({
  // plain text (aka source)
  content: "text",

  displayMode: {type: "string", optional: true},

  // volatile data derived from table
  // ATM we need it as we set it during import
  // TODO: we should try to remove that from the schema
  row: "number",
  col: "number",

  // value is derived from the plain content by evaluating
  // it in an interpreter
  value: { type: "string", optional: true }, // evaluated value
  valueType: { type: "string", default: 'text' },
});

Cell.static.generatedProps = ['value', 'valueType'];

module.exports = Cell;
