'use strict';

var Component = require('substance/ui/Component');
var CellComponent = require('./CellComponent');
var CellTeaserComponent = require('./CellTeaserComponent');
var $$ = Component.$$;

function ErrorCell() {
  ErrorCell.super.apply(this, arguments);
}

ErrorCell.Prototype = function() {

  var _super = Object.getPrototypeOf(this);

  this.render = function() {
    var el = _super.render.call(this);
    var node = this.props.node;
    
    el.addClass('sc-error-cell');
    el.addClass(node.displayMode);

    // Display cell teaser
    el.append($$(CellTeaserComponent, {node: node}));

    el.append(
      $$('div').addClass('se-error-message').append(node.value)
    );
    return el;
  };
};

CellComponent.extend(ErrorCell);

module.exports = ErrorCell;
