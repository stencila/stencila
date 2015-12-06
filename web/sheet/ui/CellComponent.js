'use strict';

var oo = require('substance/util/oo');
var Component = require('substance/ui/Component');
var TextPropertyEditor = require('substance/ui/TextPropertyEditor');
var $$ = Component.$$;

function CellComponent() {
  CellComponent.super.apply(this, arguments);
}

CellComponent.Prototype = function() {

  this.render = function() {
    var node = this.props.node;
    var el = $$('td');
    if (node) {
      if (node.isExpression()) {
        // for expressions always show the value
        // there should be a expression bar for editing the expressions
        el.text(node.getValue());
      } else {
        if (this.state.isEditing) {
          el.append($$(TextPropertyEditor, {
            name: node.id,
            path: [node.id, 'content'],
            commands: []
          }));
        } else {
          el.text(node.content);
        }
      }
    }
    el.on('click', this.onClick);
    return el;
  };

  this.onClick = function(e) {
    var node = this.props.node;
    e.preventDefault();
    e.stopPropagation();
    if (this.state.isEditing) {
      return;
    }

    if (node.isExpression()) {
      // request a state change so that the expression bar will be shown.;
      this.send('edit:expression', this.props.node.id);
    }
    this.setState({ isEditing: true });
  };

};

oo.inherit(CellComponent, Component);

module.exports = CellComponent;
