'use strict';

var oo = require('substance-fe0ed/util/oo');
var Component = require('substance-fe0ed/ui/Component');

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

  this.isEditable = function() {
    return this.context.config.isEditable;
  };

  this.onPropertiesChanged = function() {
    this.rerender();
  };

};

oo.inherit(StencilNodeComponent, Component);

module.exports = StencilNodeComponent;