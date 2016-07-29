'use strict';

var Component = require('substance/ui/Component');

var math = require('../../../shared/math');

function MathComponent() {
  MathComponent.super.apply(this, arguments);
}

MathComponent.Prototype = function() {

  this.didMount = function() {
    this.props.node.on('source:changed', this.rerender, this);
    this.props.node.on('language:changed', this.rerender, this);
    this.props.node.on('display:changed', this.rerender, this);
  };

  this.dispose = function() {
    this.props.node.off(this);
  };

  this.render = function($$) {
    var node = this.props.node;

    var el = $$('span')
      .addClass('sc-math sm-'+node.language)
      .ref('math');

    if (node.display === 'block') {
      el.addClass('sm-block');
    }

    try {
      el.html(
        math.render(node.source, node.language, node.display)
      );
    } catch(error) {
      el.addClass('sm-error')
        .append(error.message)
      ;
    }

    return el;
  };

};

Component.extend(MathComponent);

module.exports = MathComponent;
