'use strict';

var oo = require('substance/util/oo');
var Component = require('substance/ui/Component');
var TextProperty = require('substance/ui/TextPropertyComponent');
var $$ = Component.$$;
var StencilNodeComponent = require('../../StencilNodeComponent');

function StencilFigureComponent() {
  StencilNodeComponent.apply(this, arguments);
}

StencilFigureComponent.Prototype = function() {

  this.render = function() {
    var el = $$('div')
      .addClass("sc-stencil-figure content-node figure clearfix "+this.props.node.type)
      .attr("data-id", this.props.node.id);

    el.append($$('div')
      .addClass('label').attr("contenteditable", false)
      .append(this.props.node.label)
      .key('label')
    );

    el.append($$('div')
      .addClass('source')
      .append(
        $$(TextProperty, {
          tagName: 'div',
          path: [this.props.node.id, "source"]
        })
      )
      .key('source')
    );

    el.append($$('div')
      .addClass('figure-content')
      .attr('contenteditable', false)
      .append(
        $$('img')
          .addClass('image')
          .attr({
            contentEditable: false,
            src: this.props.node.getDocument().url + "/" + this.props.node.image
          })
      )
      .key('content')
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
      .key('description')
    );
    return el;
  };

};

oo.inherit(StencilFigureComponent, StencilNodeComponent);

module.exports = StencilFigureComponent;
