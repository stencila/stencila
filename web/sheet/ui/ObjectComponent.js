'use strict';

var Component = require('substance/ui/Component');
var CellContentComponent = require('./CellContentComponent');
var CellTeaserComponent = require('./CellTeaserComponent');
var $$ = Component.$$;

function ObjectComponent() {
  ObjectComponent.super.apply(this, arguments);
}

ObjectComponent.Prototype = function() {

  this.render = function() {
    var node = this.props.node;
    var el = $$('div').addClass('sc-cell-content sc-object');
    el.addClass(node.displayMode);

    var isPrimitiveType = ['string', 'int', 'real'].indexOf(node.valueType)>=0;

    if (isPrimitiveType) {
      el.text(node.value);
    } else {
      // Display cell teaser
      el.append($$(CellTeaserComponent, {node: node}));

      if (node.displayMode != 'clipped') {
        el.append($$('pre').text(node.value));
      }
    }

    return el;
  };
};

CellContentComponent.extend(ObjectComponent);

module.exports = ObjectComponent;
