"use strict";

var DocumentNode = require('substance/model/DocumentNode');

function Cell(){
  Cell.super.apply(this, arguments);
}

Cell.Prototype = function() {
  this.isExpression = function() {
    return /^\s*=/.exec(this.content);
  };
  this.getValue = function() {
    if (this.isExpression()) {
      return this.value;
    } else {
      return this.content;
    }
  };
};

DocumentNode.extend(Cell);

Cell.static.name = "sheet-cell";

Cell.static.defineSchema({
  content: "text",
  alias: { type: "string", optional: true },
  value: { type: "string", optional: true },
  row: "number",
  col: "number"
});

Cell.static.generatedProps = ['value'];

module.exports = Cell;
