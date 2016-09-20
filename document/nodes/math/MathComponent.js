'use strict';

import Component from 'substance/ui/Component'

var math = require('../../../utilities/math');

function MathComponent () {
  MathComponent.super.apply(this, arguments);
}

MathComponent.Prototype = function () {
  this.didMount = function () {
    this.props.node.on('source:changed', this.rerender, this);
    this.props.node.on('language:changed', this.rerender, this);
    this.props.node.on('display:changed', this.rerender, this);
  };

  this.dispose = function () {
    this.props.node.off(this);
  };

  this.render = function ($$) {
    var node = this.props.node;

    var el = $$('span')
      .addClass('sc-math sm-' + node.language)
      .ref('math');

    try {
      el.html(
        math.render(node.source, node.language, node.display)
      );
    } catch (error) {
      el.addClass('sm-error')
        .text(error.message);
    }

    if (node.display === 'block') {
      el.addClass('sm-block');
    }

    return el;
  };
};

Component.extend(MathComponent);

module.exports = MathComponent;
