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
    var node = this.props.node;
    var el = $$('div')
      .addClass('stencil-parameter')
      .append(
        $$('div')
          .addClass('name')
          .append($$(TextProperty, {
            path: [ node.id, "name"]
          })),
        $$('div')
          .addClass('value')
          .append($$(TextProperty, {
            path: [ node.id, "value"]
          }))
      );
    if(node.error){
      el.append(
        $$('div')
          .addClass('error')
          .text(node.error)
      );
    }
    return el;
  };

};

oo.inherit(StencilParameterComponent, StencilNodeComponent);

module.exports = StencilParameterComponent;
