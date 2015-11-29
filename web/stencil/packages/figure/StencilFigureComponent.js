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
      .addClass('stencil-figure')
      .attr("data-id", this.props.node.id);

    el.append(
      $$('div')
        .addClass('header')
        .append(
          $$('span')
            .addClass('label')
            .attr("contenteditable", false)
            .append('Figure '+this.props.node.index)
            .ref('label'),
          /*
          FIXME: 
          This is causing the error "Property already registered."
          when the stencil is rendered. The following non-editable
          span is a temporary replacement

          $$(TextProperty, {
            tagName: 'span',
            path: [this.props.node.id, 'caption']
          })
            .addClass('caption')
          */
         $$('span')
            .addClass('caption')
            .attr("contenteditable", false)
            .append(this.props.node.caption)
            .ref('caption')
        )
    );

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

    return el;
  };


};

oo.inherit(StencilFigureComponent, StencilNodeComponent);

module.exports = StencilFigureComponent;
