'use strict';

var oo = require('substance/util/oo');
var Component = require('substance/ui/Component');
var $$ = Component.$$;
var TextProperty = require('substance/ui/TextPropertyComponent');
var StencilNodeComponent = require('../../StencilNodeComponent');

function StencilExecComponent() {
  StencilNodeComponent.apply(this, arguments);
}

StencilExecComponent.Prototype = function() {

  this.getClassNames = function() {
    return "sc-stencil-exec";
  };

  this.render = function() {
    return $$('div')
      .addClass(this.getClassNames())
      .attr("data-id", this.props.node.id)
      .append($$(TextProperty, {
        path: [ this.props.node.id, "source"]
      }));
  };
};

oo.inherit(StencilExecComponent, StencilNodeComponent);

module.exports = StencilExecComponent;
