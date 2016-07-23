'use strict';

var oo = require('substance-fe0ed/util/oo');
var Component = require('substance-fe0ed/ui/Component');
var Icon = require('substance-fe0ed/ui/FontAwesomeIcon');
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
