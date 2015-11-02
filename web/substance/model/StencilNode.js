'use strict';

var OO = require('substance/util/oo');
var DocumentNode = require('substance/model/DocumentNode');
var _ = require('substance/util/helpers');

function StencilNode() {
  DocumentNode.apply(this, arguments);
}

StencilNode.Prototype = function() {

  this.updateGeneratedProperties = function(props) {
    var propNames = this.constructor.static.generatedProps;
    if (propNames) {
      _.each(propNames, function(propName) {
        this[propName] = props[propName];
      }, this);
      this.emit('properties:changed');
    }
  };
};

OO.inherit(StencilNode, DocumentNode);

StencilNode.static.generatedProps = [];

module.exports = StencilNode;
