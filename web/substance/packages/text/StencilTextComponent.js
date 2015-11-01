'use strict';

var oo = require('substance/util/oo');
var AnnotationComponent = require('substance/ui/AnnotationComponent');
var Component = require('substance/ui/Component');
var $$ = Component.$$;

function StencilTextComponent() {
  AnnotationComponent.apply(this, arguments);

  // this.props.node.connect(this, {
  //   "label:changed": this.onLabelChanged
  // });
}

StencilTextComponent.Prototype = function() {

  this.dispose = function() {
    AnnotationComponent.prototype.dispose.call(this);
    this.props.node.disconnect(this);
  };

  this.render = function() {
    return $$('span')
      .addClass(this.getClassNames())
      .attr({
        "data-id": this.props.node.id,
        "data-external": 1,
        "contentEditable": false
      })
      .on('click', this.onClick)
      .on('mousedown', this.onMouseDown)
      .append(this.props.node.output || "");
  };

  this.getClassNames = function() {
    var classNames = this.props.node.getTypeNames().join(' ');
    if (this.props.classNames) {
      classNames += " " + this.props.classNames.join(' ');
    }
    if (this.props.node.highlighted) {
      classNames += ' highlighted';
    }
    return classNames.replace(/_/g, '-');
  };

  this.onMouseDown = function(e) {
    e.preventDefault();
    e.stopPropagation();
    var node = this.props.node;
    var surface = this.context.surface;

    surface.setSelection(node.getSelection());
    var controller = this.context.controller;
    controller.emit('edit:source', node);
  };

  this.onClick = function(e) {
    e.preventDefault();
    e.stopPropagation();
  };

  // this.onLabelChanged = function() {
  //   this.rerender();
  // };
};

oo.inherit(StencilTextComponent, AnnotationComponent);

module.exports = StencilTextComponent;