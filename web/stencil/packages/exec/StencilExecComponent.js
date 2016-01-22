'use strict';

var oo = require('substance/util/oo');
var extend = require('lodash/object/extend');
var Component = require('substance/ui/Component');
var $$ = Component.$$;
var TextProperty = require('substance/ui/TextPropertyComponent');
var StencilNodeComponent = require('../../StencilNodeComponent');
var StencilSourceComponent = require('../../StencilSourceComponent');
var Icon = require('substance/ui/FontAwesomeIcon');

function StencilExecComponent() {
  StencilNodeComponent.apply(this, arguments);
}

StencilExecComponent.Prototype = function() {
  extend(this, StencilSourceComponent.prototype);

  this.getClassNames = function() {
    return "stencil-exec";
  };

  this.render = function() {
    var el = $$('div')
      .addClass(this.getClassNames())
      .attr("data-id", this.props.node.id)
      .attr("contentEditable", false);

    if (this.isEditable()) {
      var button = $$('button')
          .append(
            $$(Icon, {icon: 'fa-flash'})
          )
          // Bind click; we need to suppress mouse down, as otherwise
          // Surface will receive events leading to updating the selection
          .on('click', this.onEditSource)
          .on('mousedown', this.onMouseDown);
      el.append(
        button
      );
      if (this.props.node.error) {
        button.addClass('error');
      }
    }

    if (this.revealSource()) {
      el.append(
        $$(TextProperty, {
          tagName: 'div',
          path: [ this.props.node.id, "source"]
        })
        .addClass('se-exec-source')
        .ref('source')
      );
    }

    if (this.props.node.error) {
      el.addClass('sm-error');
    }

    return el;
  };

};

oo.inherit(StencilExecComponent, StencilNodeComponent);

module.exports = StencilExecComponent;
