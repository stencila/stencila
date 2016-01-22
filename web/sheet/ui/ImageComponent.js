'use strict';

var Component = require('substance/ui/Component');
var CellContentComponent = require('./CellContentComponent');
var CellTeaserComponent = require('./CellTeaserComponent');
var $$ = Component.$$;

function ImageComponent() {
  ImageComponent.super.apply(this, arguments);
}

ImageComponent.Prototype = function() {

  this.render = function() {
    var node = this.props.node;
    var el = $$('div').addClass('sc-cell-content sc-image');
    el.addClass(node.displayMode);

    // Display cell teaser
    el.append($$(CellTeaserComponent, {node: node}));

    if (node.displayMode != 'clipped') {
      el.append(
        $$('img').attr('src', node.value)
      );
    }

    return el;
  };
};

CellContentComponent.extend(ImageComponent);

module.exports = ImageComponent;
