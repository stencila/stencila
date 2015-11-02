'use strict';

var oo = require('substance/util/oo');
var Component = require('substance/ui/Component');

function StencilNodeComponent() {
  Component.apply(this, arguments);
}

StencilNodeComponent.Prototype = function() {

  this.didMount = function() {
    this.props.node.connect(this, { 'properties:changed': this.onPropertiesChanged });
  };

  this.dispose = function() {
    this.props.node.disconnect(this);
  };

  this.revealSource = function() {
    var controller = this.context.controller;
    return controller.state.revealSource;    
  };

  this.onPropertiesChanged = function() {
    this.rerender();
  };

};

oo.inherit(StencilNodeComponent, Component);

module.exports = StencilNodeComponent;