'use strict';

var Component = require('substance/ui/Component');

function PrintComponent () {
  PrintComponent.super.apply(this, arguments);
}

PrintComponent.Prototype = function () {
  this.didMount = function () {
    this.props.node.on('content:changed', this.rerender, this);
  };

  this.dispose = function () {
    this.props.node.off(this);
  };

  this.render = function ($$) {
    var node = this.props.node;
    return $$('span')
      .addClass('sc-print' + (node.error ? ' sm-error' : ''))
      .append(node.content.length ? node.content : ' ');
  };
};

Component.extend(PrintComponent);

module.exports = PrintComponent;
