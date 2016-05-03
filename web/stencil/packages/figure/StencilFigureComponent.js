'use strict';

var oo = require('substance/util/oo');
var extend = require('lodash/object/extend');
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
  extend(this, StencilSourceComponent.prototype);

  this.render = function() {
    var node = this.props.node;

    var el = $$('div')
      .addClass('stencil-figure')
      .attr("data-id", node.id);

    var icon = $$(Icon, {icon: 'fa-bar-chart'});
    icon.on('click', this.onEditSource)
        .on('mousedown', this.onMouseDown);
    el.append(icon);

    if(node.caption){
        var header = $$('div').addClass('header');
        el.append(header);
        if(node.index){
          var label = $$('span')
                .addClass('label')
                .attr("contenteditable", false)
                .append('Figure '+node.index)
                .ref('label');
          header.append(label);
        }
        header.append(
            /*
            FIXME:
            This is causing the error "Property already registered."
            when the stencil is rendered. The following non-editable
            span is a temporary replacement

            $$(TextProperty, {
              tagName: 'span',
              path: [node.id, 'caption']
            })
              .addClass('caption')
            */
           $$('span')
              .addClass('caption')
              .attr("contenteditable", false)
              .append(node.caption)
              .ref('caption')
        );
    }

    el.append($$('div')
      .addClass('content')
      .attr('contenteditable', false)
      .append(
        $$('img')
          .addClass('image')
          .attr({
            contentEditable: false,
            src: node.image,
            style: node.image_style
          })
      )
      .ref('content')
      .on('click', this.onEditSource)
      .on('mousedown', this.onMouseDown)
    );

    if (node.error) {
      el.addClass('error');
    }

    return el;
  };


};

oo.inherit(StencilFigureComponent, StencilNodeComponent);

module.exports = StencilFigureComponent;
