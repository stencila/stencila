'use strict';

var Component = require('substance/ui/Component');
var CellTeaserComponent = require('./CellTeaserComponent');
var $$ = Component.$$;

function ObjectComponent() {
  ObjectComponent.super.apply(this, arguments);
}

ObjectComponent.Prototype = function() {

  this.render = function() {
    var el = $$('div').addClass('sc-object');
    el.addClass(this.props.displayMode);

    // Display cell teaser
    el.append($$(CellTeaserComponent, {node: this.props.node}));

    if (this.props.displayMode != 'clipped') {
      el.append($$('pre').text(this.node.value));
    }
    return el;
  };
};

Component.extend(ObjectComponent);

module.exports = ObjectComponent;
