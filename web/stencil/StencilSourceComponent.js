'use strict';

var oo = require('substance/util/oo');

// Mix-in for components of nodes which have an editable source property.
function StencilSourceComponent() {}

StencilSourceComponent.Prototype = function() {

  this.onEditSource = function(e) {
    e.preventDefault();
    e.stopPropagation();
    this.send('switchState', {
      contextId: 'edit-source',
      nodeId: this.props.node.id
    });
  };

  this.onMouseDown = function(e) {
    e.preventDefault();
    e.stopPropagation();
  };

};

oo.initClass(StencilSourceComponent);

module.exports = StencilSourceComponent;
