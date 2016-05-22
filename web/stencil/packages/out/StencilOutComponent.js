'use strict';

var oo = require('substance/util/oo');
var Component = require('substance/ui/Component');
var Icon = require('substance/ui/FontAwesomeIcon');
var $$ = Component.$$;
var StencilNodeComponent = require('../../StencilNodeComponent');

function StencilOutComponent() {
  StencilNodeComponent.apply(this, arguments);
}

StencilOutComponent.Prototype = function() {

  this.render = function() {
    var node = this.props.node;

    var el = $$('div')
      .addClass('stencil-out')
      .attr("data-id", node.id)
      .attr("contenteditable", false)
      .html(node.content);
  
    return el;
  };

};

oo.inherit(StencilOutComponent, StencilNodeComponent);

module.exports = StencilOutComponent;
