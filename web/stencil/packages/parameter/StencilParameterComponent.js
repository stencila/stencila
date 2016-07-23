'use strict';

var oo = require('substance-fe0ed/util/oo');
var Component = require('substance-fe0ed/ui/Component');
var $$ = Component.$$;
var StencilNodeComponent = require('../../StencilNodeComponent');
var TextProperty = require('substance-fe0ed/ui/TextPropertyComponent');


function StencilParameterComponent() {
  StencilNodeComponent.apply(this, arguments);
}

StencilParameterComponent.Prototype = function() {

  this.render = function() {
    var node = this.props.node;
    var label = node.label || node.name;
    var el = $$('div')
      .addClass('stencil-parameter')
      .append(
        $$('div')
          .addClass('name')
          // TODO Temporarily make this read-only
          // but ultimately both name and label
          // should be able to be edited independently
          .text(label)
          .attr("contenteditable", false)
          /*.append($$(TextProperty, {
            path: [ node.id, "name"]
          }))*/,
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
