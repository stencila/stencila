/* global MathJax */
'use strict';

var oo = require('substance/util/oo');
var extend = require('lodash/object/extend');
var Component = require('substance/ui/Component');
var TextProperty = require('substance/ui/TextPropertyComponent');
var $$ = Component.$$;

var StencilNodeComponent = require('../../StencilNodeComponent');
var StencilSourceComponent = require('../../StencilSourceComponent');

function StencilEquationComponent() {
  StencilNodeComponent.apply(this, arguments);
}

StencilEquationComponent.Prototype = function() {
  extend(this, StencilSourceComponent.prototype);

  this._renderMathJax = function() {
    var node = this.props.node;
    var tagName = this.props.node.constructor.static.isInline ? 'span' : 'div';
    var typeAttr = node.format;
    if (!node.constructor.static.isInline) {
      typeAttr += "; mode=display";
    }
    return $$(tagName)
      .addClass('se-math-output')
      .ref('output')
      // appending the script here as MJ then renders a preview
      .append(
        $$('script')
          .attr('type', typeAttr)
          .append(node.source)
          .ref('source')
      )
      .append($$(tagName)
        .addClass('se-error')
        .ref('error')
      );
  };

  this.render = function() {
    var node = this.props.node;
    var el = $$('div')
      .addClass("stencil-equation " + node.type)
      .attr("data-id", node.id)
      .attr('contentEditable', false)
      .on('click', this.onEditSource)
      .on('mousedown', this.onMouseDown);

    el.append(this._renderMathJax());

    if (this.revealSource()) {
      el.append(
        $$(TextProperty, {
            tagName: 'div',
            path: [node.id, "source"]
          })
          .addClass('source')
        )
        .ref('source');
    }
    return el;
  };

  this.didMount = function() {
    StencilNodeComponent.prototype.didMount.call(this);
    MathJax.Hub.Queue(["Process", MathJax.Hub,this.refs.source.el]);
    this.props.node.connect(this, {
      'source:changed': this.rerenderMath
    });
  };

  this.rerenderMath = function() {
    var node = this.props.node;
    var scriptComp = this.refs.source;
    scriptComp.$el.text(node.source);
    MathJax.Hub.Queue(["Reprocess", MathJax.Hub, scriptComp.el]);
  };

  this.onParseError = function(message) {
    console.error("MathJax error", message);
    this.refs.error.$el.text(message[1]);
  };

};

oo.inherit(StencilEquationComponent, StencilNodeComponent);

module.exports = StencilEquationComponent;
