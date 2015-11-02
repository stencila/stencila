'use strict';

var oo = require('substance/util/oo');
var Component = require('substance/ui/Component');
var $$ = Component.$$;
var TextProperty = require('substance/ui/TextPropertyComponent');
var StencilNodeComponent = require('../../StencilNodeComponent');

function StencilExecComponent() {
  StencilNodeComponent.apply(this, arguments);
}

StencilExecComponent.Prototype = function() {

  this.getClassNames = function() {
    return "sc-stencil-exec";
  };

  this.render = function() {
    console.log('StencilExecComponent.render()');
    var el = $$('div')
      .addClass(this.getClassNames())
      .attr("data-id", this.props.node.id);

    if (this.isEditable()) {
      
    }
    
    if (this.revealSource()) {
      el.append(
        $$(TextProperty, {
          tagName: 'div',
          path: [ this.props.node.id, "source"]
        }).addClass('se-exec-source')
      );
    }
    return el;
  };
};

oo.inherit(StencilExecComponent, StencilNodeComponent);

module.exports = StencilExecComponent;
