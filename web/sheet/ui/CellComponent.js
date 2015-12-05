'use strict';

var oo = require('substance/util/oo');
var Component = require('substance/ui/Component');
var $$ = Component.$$;

function CellComponent() {
  CellComponent.super.apply(this, arguments);
}

CellComponent.Prototype = function() {

  this.render = function() {
    var node = this.props.node;
    var el = $$('td');
    if (node) {
      if (this.state.isEditing) {
        el.text(node.content);
      } else {
        el.text(node.getValue());
      }
    }
    return el;
  };

  this.onClick = function(e) {
    var node = this.props.node;
    e.preventDefault();
    e.stopPropagation();
    if (node) {
      console.log('Clicked on cell (%s, %s)', node.row, node.col);
    } else {
      console.log('Clicked on empty cell.');
    }
  };

};

oo.inherit(CellComponent, Component);

module.exports = CellComponent;
