'use strict';

var oo = require('substance/util/oo');
var _ = require('substance/util/helpers');
var AnnotationComponent = require('substance/ui/AnnotationComponent');
var Component = require('substance/ui/Component');
var $$ = Component.$$;
var StencilNodeComponent = require('../../StencilNodeComponent');
var StencilSourceComponent = require('../../StencilSourceComponent');
var StencilEquationComponent = require('./StencilEquationComponent');

function StencilFormulaComponent() {
  AnnotationComponent.apply(this, arguments);
}

StencilFormulaComponent.Prototype = function() {

  // use StencilNodeComponent as a mixin
  _.extend(this, StencilNodeComponent.prototype);

  _.extend(this, StencilSourceComponent.prototype);

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
      .addClass(this.getClassNames() + " sc-formula")
      .attr({
        "data-id": this.props.node.id,
        "data-external": 1,
        "contentEditable": false
      })
      .on('click', this.onEditSource)
      .on('mousedown', this.onMouseDown)

    el.append(StencilEquationComponent.prototype._renderMathJax.call(this));
    return el;
  };

  this.rerenderMath = StencilEquationComponent.prototype.rerenderMath;

};

oo.inherit(StencilFormulaComponent, AnnotationComponent);

module.exports = StencilFormulaComponent;
