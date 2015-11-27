'use strict';

var oo = require('substance/util/oo');
var Component = require('substance/ui/Component');
var $$ = Component.$$;
var TextProperty = require('substance/ui/TextPropertyComponent');

function StencilSummaryComponent() {
  Component.apply(this, arguments);
}

StencilSummaryComponent.Prototype = function() {

  this.render = function() {
    return $$('div')
      .addClass('stencil-summary')
      .attr("data-id", this.props.node.id)
      .append($$(TextProperty, {
        path: [ this.props.node.id, "content"]
      }));
  };

};

oo.inherit(StencilSummaryComponent, Component);

module.exports = StencilSummaryComponent;
