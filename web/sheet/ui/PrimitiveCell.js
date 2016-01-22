'use strict';

var Component = require('substance/ui/Component');
var CellComponent = require('./CellComponent');
var CellTeaserComponent = require('./CellTeaserComponent');
var $$ = Component.$$;

function PrimitiveCell() {
  PrimitiveCell.super.apply(this, arguments);
}

PrimitiveCell.Prototype = function() {
  this.render = function() {
    var el = PrimitiveCell.super.prototype.render.call(this);
    var node = this.props.node;
    
    el.addClass('sc-primitive-cell');
    
    if (!this.isEditing()) {
      el.addClass(node.displayMode);
      el.append(node.value);
    }

    // TODO: maybe introduce displaymode expanded to show source also
    return el;
  };
};

CellComponent.extend(PrimitiveCell);
module.exports = PrimitiveCell;
