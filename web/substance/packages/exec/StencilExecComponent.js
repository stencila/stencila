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
    var el = $$('div')
      .addClass(this.getClassNames())
      .attr("data-id", this.props.node.id)
      .attr("contentEditable", false);

    if (this.isEditable()) {
      el.append(
        $$('button')
          .addClass('se-exec-button')
          .append(
            $$('span').addClass('se-label').append(this.i18n.t('exec-button-label')),
            $$('span').addClass('se-action').append(this.i18n.t('edit-source-action'))
          )
          .on('click', this.onClickEdit)
          // Unfortunately we need to suppress mouse down, as otherwise
          // Surface will receive events leading to updating the selection
          .on('mousedown', this.onMouseDown)
      );
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

  this.onClickEdit = function(e) {
    e.preventDefault();
    e.stopPropagation();
    this.send('switchState', {
      contextId: 'editSource',
      nodeId: this.props.node.id
    });
  };

  this.onMouseDown = function(e) {
    e.preventDefault();
    e.stopPropagation();
  };

};

oo.inherit(StencilExecComponent, StencilNodeComponent);

module.exports = StencilExecComponent;
