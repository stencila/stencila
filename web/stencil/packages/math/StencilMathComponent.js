'use strict';

var oo = require('substance/util/oo');
var extend = require('lodash/object/extend');
var AnnotationComponent = require('substance/ui/AnnotationComponent');
var Component = require('substance/ui/Component');
var $$ = Component.$$;

var StencilNodeComponent = require('../../StencilNodeComponent');
var StencilSourceComponent = require('../../StencilSourceComponent');
var StencilEquationComponent = require('../equation/StencilEquationComponent');

function StencilMathComponent() {
  AnnotationComponent.apply(this, arguments);
}

StencilMathComponent.Prototype = function() {

  // mix-ins
  extend(this, StencilNodeComponent.prototype);
  extend(this, StencilSourceComponent.prototype);

  this.didMount = function() {
    AnnotationComponent.prototype.didMount.call(this);
    StencilEquationComponent.prototype.didMount.call(this);
  };

  this.dispose = function() {
    AnnotationComponent.prototype.dispose.call(this);
    StencilNodeComponent.prototype.dispose.call(this);
  };

  this.render = function() {
    var el = $$('span')
      .addClass(this.getClassNames() + " stencil-math")
      .attr({
        "data-id": this.props.node.id,
        "data-external": 1,
        "contentEditable": false
      })
      .on('click', this.onEditSource)
      .on('mousedown', this.onMouseDown);

    el.append(StencilEquationComponent.prototype._renderMathJax.call(this));
    return el;
  };

  this.rerenderMath = StencilEquationComponent.prototype.rerenderMath;

};

oo.inherit(StencilMathComponent, AnnotationComponent);

module.exports = StencilMathComponent;
