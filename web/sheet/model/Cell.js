"use strict";

var DocumentNode = require('substance/model/DocumentNode');

function Cell(){
  Cell.super.apply(this, arguments);
}

Cell.Prototype = function() {
  this.isExpression = function() {
    var match = /^\s*=/.exec(this.content);
    return !!match;
  };
  this.getValue = function() {
    if (this.isExpression()) {
      return this.value;
    } else {
      return this.content;
    }
  };
  this.getExpression = function() {
    var match = /^\s*=\s*(.+)$/.exec(this.content);
    if (match) {
      return match[1];
    } else {
      return null;
    }
  };
};

DocumentNode.extend(Cell);

Cell.static.name = "sheet-cell";

Cell.static.defineSchema({
  content: "text",
  name: { type: "string", optional: true },
  value: { type: "string", optional: true },
  row: "number",
  col: "number"
});

Cell.static.generatedProps = ['value'];

module.exports = Cell;
