'use strict';

var Component = require('substance/ui/Component');
var CellComponent = require('./CellComponent');
var CellTeaserComponent = require('./CellTeaserComponent');
var $$ = Component.$$;

function ImageCell() {
  ImageCell.super.apply(this, arguments);
}

ImageCell.Prototype = function() {

  var _super = Object.getPrototypeOf(this);

  this.render = function() {
    var el = _super.render.call(this);
    var node = this.props.node;
    
    el.addClass('sc-image-cell');
    el.addClass(node.displayMode);

    if (!this.isEditing()) {
      // Display cell teaser
      el.append($$(CellTeaserComponent, {node: node}));

      if (node.displayMode != 'clipped') {
        el.append(
          $$('img').attr('src', node.value)
        );
      }
    }
    return el;
  };
};

CellComponent.extend(ImageCell);

module.exports = ImageCell;
