'use strict';

var oo = require('substance/util/oo');
var _ = require('substance/util/helpers');
var Component = require('substance/ui/Component');
var TextProperty = require('substance/ui/TextPropertyComponent');
var Icon = require('substance/ui/FontAwesomeIcon');
var $$ = Component.$$;
var StencilNodeComponent = require('../../StencilNodeComponent');
var StencilSourceComponent = require('../../StencilSourceComponent');

function StencilFigureComponent() {
  StencilNodeComponent.apply(this, arguments);
}

StencilFigureComponent.Prototype = function() {

  _.extend(this, StencilSourceComponent.prototype);

  this.render = function() {
    var el = $$('div')
      .addClass("sc-stencil-figure content-node figure clearfix "+this.props.node.type)
      .attr("data-id", this.props.node.id)
      .attr('id', this.props.node.id);

    el.append($$('div')
      .addClass('label').attr("contenteditable", false)
      .append(this.props.node.label)
      .ref('label')
    );

    if (this.isEditable()) {
      el.append(
        $$('button')
          .addClass('se-figure-edit-button')
          .append(
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
      el.append($$('div')
        .addClass('source')
        .append(
          $$(TextProperty, {
            tagName: 'div',
            path: [this.props.node.id, "source"]
          })
        )
        .ref('source')
      );
    }

    el.append($$('div')
      .addClass('figure-content')
      .attr('contenteditable', false)
      .append(
        $$('img')
          .addClass('image')
          .attr({
            contentEditable: false,
            src: this.props.node.image
          })
      )
      .ref('content')
    );

    el.append($$('div')
      .addClass('description small')
      .append(
        $$(TextProperty, {
          tagName: 'div',
          path: [this.props.node.id, "caption"]
        })
        .addClass('caption')
      )
      .ref('description')
    );

    if (this.props.node.error) {
      el.addClass('sm-error');
    }
    return el;
  };


};

oo.inherit(StencilFigureComponent, StencilNodeComponent);

module.exports = StencilFigureComponent;
