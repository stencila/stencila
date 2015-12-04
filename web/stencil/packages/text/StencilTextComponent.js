'use strict';

var oo = require('substance/util/oo');
var _ = require('substance/util/helpers');
var AnnotationComponent = require('substance/ui/AnnotationComponent');
var StencilNodeComponent = require('../../StencilNodeComponent');
var Component = require('substance/ui/Component');
var $$ = Component.$$;

function StencilTextComponent() {
  AnnotationComponent.apply(this, arguments);
}

StencilTextComponent.Prototype = function() {
  _.extend(this, StencilNodeComponent.prototype);

  this.didMount = function() {
    AnnotationComponent.prototype.didMount.call(this);
    StencilNodeComponent.prototype.didMount.call(this);
  };

  this.dispose = function() {
    AnnotationComponent.prototype.dispose.call(this);
    StencilNodeComponent.prototype.dispose.call(this);
  };

  this.render = function() {
    var el;

    if (this.isEditable()) {
      el = $$('span')
        .addClass(this.getClassNames())
        .attr({
          "data-id": this.props.node.id,
          "data-external": 1,
          "contentEditable": false
        })
        .on('click', this.onClick)
        .on('mousedown', this.onMouseDown)
        .append(this.props.node.output || "?");
    } else {
      el = $$('span')
        .addClass('stencil-text');

      if (this.revealSource()) {
        el.append(
          $$('span').addClass('source').append(this.props.node.source),
          ' → '
        );
      }
      el.append(
        this.props.node.output
      );
    }

    if (this.props.node.error) {
      el.addClass('error');
    }

    return el;
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

  this.dispose = function() {
    AnnotationComponent.prototype.dispose.call(this);
  };

};

oo.inherit(StencilTextComponent, AnnotationComponent);

module.exports = StencilTextComponent;
