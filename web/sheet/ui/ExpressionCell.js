'use strict';

var Component = require('substance/ui/Component');
var CellComponent = require('./CellComponent');
var CellTeaserComponent = require('./CellTeaserComponent');
var $$ = Component.$$;

/**
  Displays expression cells, such that start with '=' and are
  not handled by a specific component.
*/

function ExpressionCell() {
  ExpressionCell.super.apply(this, arguments);
}

ExpressionCell.Prototype = function() {
  var _super = Object.getPrototypeOf(this);

  this.render = function() {
    var el = _super.render.call(this);
    var node = this.props.node;
    
    el.addClass('sc-expression-cell');
    el.addClass(node.displayMode);

    // Display cell teaser
    el.append($$(CellTeaserComponent, {node: node}));

    if (node.displayMode != 'clipped') {
      el.append($$('pre').text(node.value));
    }
    return el;
  };
};

CellComponent.extend(ExpressionCell);

module.exports = ExpressionCell;
