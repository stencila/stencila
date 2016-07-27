'use strict';

var Component = require('substance/ui/Component');

function Print() {
  Print.super.apply(this, arguments);
}

Print.Prototype = function() {

  this.didMount = function() {
    this.props.node.on('source:changed', this.rerender, this);
  };

  this.dispose = function() {
    this.props.node.off(this);
  };

  this.render = function($$) {
    var node = this.props.node;
    var el = $$('span')
      .addClass('sc-print')
      .append(node.content.length ? node.content : ' ');
    return el;
  };

};

Component.extend(Print);

module.exports = Print;
