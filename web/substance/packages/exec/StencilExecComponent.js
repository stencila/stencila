'use strict';

var oo = require('substance/util/oo');
var _ = require('substance/util/helpers');
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

  _.extend(this, StencilSourceComponent.prototype);

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
            $$('span').addClass('se-action').append(
              $$(Icon, {icon: 'fa-pencil'}),
              ' ',
              this.i18n.t('edit-source-action')
            )
          )
          .on('click', this.onEditSource)
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
