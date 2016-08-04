'use strict';

var Component = require('substance/ui/Component');

function DefaultComponent() {
 DefaultComponent.super.apply(this, arguments);
}

DefaultComponent.Prototype = function() {

  this.render = function($$) {
    return $$('div')
      .attr('contenteditable', false)
      .attr('data-id', this.props.node.id)
      .addClass('sc-default')
      .html(this.props.node.html);
  };

};

Component.extend(DefaultComponent);

module.exports = DefaultComponent;
