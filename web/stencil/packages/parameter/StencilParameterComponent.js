'use strict';

var oo = require('substance/util/oo');
var Component = require('substance/ui/Component');
var $$ = Component.$$;
var StencilNodeComponent = require('../../StencilNodeComponent');
var TextProperty = require('substance/ui/TextPropertyComponent');


function StencilParameterComponent() {
  StencilNodeComponent.apply(this, arguments);
}

StencilParameterComponent.Prototype = function() {

  this.render = function() {
    var el = $$('div')
      .addClass('stencil-parameter')
      .append(
        $$('div')
          .addClass('stencil-parameter-name')
          .append($$(TextProperty, {
            path: [ this.props.node.id, "name"]
          })),
        $$('div')
          .addClass('stencil-parameter-value')
          .append($$(TextProperty, {
            path: [ this.props.node.id, "value"]
          }))
      );
    return el;
  };

};

oo.inherit(StencilParameterComponent, StencilNodeComponent);

module.exports = StencilParameterComponent;
