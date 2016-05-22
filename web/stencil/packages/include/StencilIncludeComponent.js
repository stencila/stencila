'use strict';

var oo = require('substance/util/oo');
var extend = require('lodash/object/extend');
var Component = require('substance/ui/Component');
var $$ = Component.$$;
var Icon = require('substance/ui/FontAwesomeIcon');

var StencilNodeComponent = require('../../StencilNodeComponent');
var StencilSourceComponent = require('../../StencilSourceComponent');

function StencilIncludeComponent() {
  StencilNodeComponent.apply(this, arguments);
}

StencilIncludeComponent.Prototype = function() {

  extend(this, StencilSourceComponent.prototype);

  this.render = function() {
    var node = this.props.node;

    var el = $$('div')
      .attr("data-id", node.id)
      .addClass('stencil-include')
      .attr("contenteditable", false)
      .append($$(Icon, {icon: 'fa-arrow-circle-right'}))
      .on('click', this.onEditSource)
      .on('mousedown', this.onMouseDown)
      .append($$('div').html(node.content));

    return el;
  };

};

oo.inherit(StencilIncludeComponent, StencilNodeComponent);

module.exports = StencilIncludeComponent;
