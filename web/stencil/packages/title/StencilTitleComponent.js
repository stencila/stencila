'use strict';

var oo = require('substance/util/oo');
var Component = require('substance/ui/Component');
var $$ = Component.$$;
var TextProperty = require('substance/ui/TextPropertyComponent');

function StencilTitleComponent() {
  Component.apply(this, arguments);
}

StencilTitleComponent.Prototype = function() {

  this.render = function() {
    return $$('div')
      .addClass('sc-stencil-title')
      .attr("data-id", this.props.node.id)
      .append($$(TextProperty, {
        path: [ this.props.node.id, "content"]
      }));
  };

};

oo.inherit(StencilTitleComponent, Component);

module.exports = StencilTitleComponent;
