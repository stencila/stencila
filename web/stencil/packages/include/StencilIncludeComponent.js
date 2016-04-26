'use strict';

var oo = require('substance/util/oo');
var Component = require('substance/ui/Component');
var $$ = Component.$$;

var StencilNodeComponent = require('../../StencilNodeComponent');

function StencilIncludeComponent() {
  StencilNodeComponent.apply(this, arguments);
}

StencilIncludeComponent.Prototype = function() {

  this.render = function() {
    var node = this.props.node;

    var el = $$('div')
      .attr("data-id", node.id)
      .attr("contenteditable", false)
      .html(node.content);

    return el;
  };

};

oo.inherit(StencilIncludeComponent, StencilNodeComponent);

module.exports = StencilIncludeComponent;
