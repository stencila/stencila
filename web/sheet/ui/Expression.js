'use strict';

var Component = require('substance/ui/Component');
var CellTeaserComponent = require('./CellTeaserComponent');
var $$ = Component.$$;

/**
  Displays expression cells, such that start with '=' and are
  not handled by a specific component.
*/

function Expression() {
  Expression.super.apply(this, arguments);
}

Expression.Prototype = function() {

  this.render = function() {
    var node = this.props.node;
    var el = $$('div').addClass('sc-expression');
    // Display cell teaser
    el.append($$(CellTeaserComponent, {node: node}));
    if (node.value !== undefined && node.displayMode !== 'cli') {
      el.append(
        $$('pre').append(node.value)
      );
    }
    return el;
  };
};

Component.extend(Expression);

module.exports = Expression;
